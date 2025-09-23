#!/usr/bin/env node

/**
 * MCP Gateway HTTP Stream Test
 * 直接通过 HTTP POST 请求测试 MCP Gateway 的流式响应
 */

import fetch from 'node-fetch';

// 配置常量
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint ID
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432'; // 要测试的 agentId

async function testHttpStream() {
  console.log('🚀 开始测试 MCP Gateway HTTP Stream...');
  console.log(`📡 连接地址: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`);
  console.log(`🎯 测试接口: /bot-agent/findByAgentId`);
  console.log(`📝 AgentId: ${AGENT_ID}`);
  console.log('─'.repeat(60));

  try {
    // 1. 首先获取工具列表
    console.log('\n📋 获取工具列表...');
    const toolsRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list'
    };

    const toolsResponse = await fetch(`${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/x-ndjson'
      },
      body: JSON.stringify(toolsRequest)
    });

    if (!toolsResponse.ok) {
      throw new Error(`HTTP ${toolsResponse.status}: ${toolsResponse.statusText}`);
    }

    console.log('✅ 成功连接到流式端点');

    // 读取流式响应
    const toolsText = await toolsResponse.text();
    console.log('📊 工具列表响应:', toolsText);

    // 解析响应找到目标工具
    let toolsData;
    try {
      toolsData = JSON.parse(toolsText);
    } catch (e) {
      console.log('尝试按行解析 NDJSON...');
      const lines = toolsText.trim().split('\n');
      for (const line of lines) {
        try {
          const parsed = JSON.parse(line);
          if (parsed.result && parsed.result.tools) {
            toolsData = parsed;
            break;
          }
        } catch (lineError) {
          console.log('跳过无效行:', line);
        }
      }
    }

    if (!toolsData || !toolsData.result || !toolsData.result.tools) {
      console.error('❌ 无法解析工具列表响应');
      return;
    }

    const tools = toolsData.result.tools;
    console.log(`找到 ${tools.length} 个工具:`);
    tools.forEach((tool, index) => {
      console.log(`${index + 1}. ${tool.name} - ${tool.description || '无描述'}`);
    });

    // 查找目标工具
    const targetTool = tools.find(tool => 
      tool.name.includes('findByAgentId') || 
      tool.name.includes('bot-agent') ||
      tool.name.includes('get_bot_agent_findbyagentid')
    );

    if (!targetTool) {
      console.log('❌ 未找到 findByAgentId 相关的工具');
      console.log('可用工具:', tools.map(t => t.name));
      return;
    }

    console.log(`\n🎯 找到目标工具: ${targetTool.name}`);

    // 2. 调用目标工具
    console.log('\n🔧 调用工具获取 agent 信息...');
    const toolCallRequest = {
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: {
        name: targetTool.name,
        arguments: {
          query: {
            agentId: AGENT_ID
          }
        }
      }
    };

    const callResponse = await fetch(`${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/x-ndjson'
      },
      body: JSON.stringify(toolCallRequest)
    });

    if (!callResponse.ok) {
      throw new Error(`HTTP ${callResponse.status}: ${callResponse.statusText}`);
    }

    const callText = await callResponse.text();
    console.log('📊 工具调用原始响应:', callText);

    // 解析调用结果
    let callData;
    try {
      callData = JSON.parse(callText);
    } catch (e) {
      console.log('尝试按行解析 NDJSON...');
      const lines = callText.trim().split('\n');
      for (const line of lines) {
        try {
          const parsed = JSON.parse(line);
          if (parsed.result && parsed.id === 2) {
            callData = parsed;
            break;
          }
        } catch (lineError) {
          console.log('跳过无效行:', line);
        }
      }
    }

    if (callData && callData.result) {
      console.log('✅ 工具调用成功!');
      console.log('📋 返回结果:');
      console.log(JSON.stringify(callData.result, null, 2));

      // 如果有内容，尝试解析
      if (callData.result.content && callData.result.content.length > 0) {
        console.log('\n📊 解析后的响应内容:');
        callData.result.content.forEach((item, index) => {
          console.log(`内容 ${index + 1} (${item.type}):`);
          if (item.type === 'text') {
            console.log(item.text);
            
            // 尝试解析 JSON
            try {
              const jsonData = JSON.parse(item.text);
              console.log('解析的 JSON 数据:');
              console.log(JSON.stringify(jsonData, null, 4));
            } catch (e) {
              console.log('(非 JSON 格式数据)');
            }
          }
        });
      }
    } else {
      console.error('❌ 工具调用失败');
      if (callData && callData.error) {
        console.error('错误信息:', callData.error);
      }
    }

  } catch (error) {
    console.error('❌ 测试失败:', error);
    console.error('错误详情:', error.message);
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway HTTP Stream 集成测试');
  console.log('═'.repeat(60));
  
  try {
    await testHttpStream();
  } catch (error) {
    console.error('主程序执行失败:', error);
    process.exit(1);
  }
  
  console.log('\n🏁 测试完成');
}

// 运行测试
main().catch(console.error);