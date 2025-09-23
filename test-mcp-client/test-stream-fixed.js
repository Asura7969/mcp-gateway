#!/usr/bin/env node

/**
 * MCP Gateway HTTP Stream 测试 - 修复版本
 * 专门测试 findByAgentId 接口
 */

import fetch from 'node-fetch';

// 配置
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432';

async function testHttpStream() {
  try {
    console.log('🚀 开始测试 MCP Gateway HTTP Stream...');
    console.log(`📡 连接地址: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`);
    console.log(`🎯 测试接口: /bot-agent/findByAgentId`);
    console.log(`📝 AgentId: ${AGENT_ID}`);
    console.log('─'.repeat(60));

    // 1. 获取工具列表
    console.log('\n📋 获取工具列表...');
    const toolsRequest = {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list',
      params: {}
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
    const toolsText = await toolsResponse.text();
    console.log('📊 工具列表响应:', toolsText);

    // 解析工具列表
    const toolsData = JSON.parse(toolsText);
    const tools = toolsData.result.tools;

    console.log(`找到 ${tools.length} 个工具:`);
    tools.forEach((tool, index) => {
      console.log(`${index + 1}. ${tool.name} - ${tool.description}`);
    });

    // 明确查找 findByAgentId 工具（GET请求）
    const targetTool = tools.find(tool => 
      tool.name.includes('get_bot-agent_findByAgentId') ||
      (tool.name.includes('findByAgentId') && tool.name.startsWith('get_'))
    );

    if (!targetTool) {
      console.log('❌ 未找到 findByAgentId 相关的GET工具');
      console.log('可用工具:', tools.map(t => t.name));
      return;
    }

    console.log(`\n🎯 找到目标工具: ${targetTool.name}`);
    console.log('🔍 工具详情:', JSON.stringify(targetTool, null, 2));

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

    console.log('📤 发送请求:', JSON.stringify(toolCallRequest, null, 2));

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

    // 解析NDJSON响应
    const lines = callText.trim().split('\n');
    console.log(`\n📋 解析 ${lines.length} 行响应:`);

    let finalResult = null;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      try {
        const parsed = JSON.parse(line);
        console.log(`\n行 ${i + 1}: ${parsed.result?.type || 'result'}`);
        
        if (parsed.result?.type === 'progress') {
          console.log(`⏳ 进度: ${parsed.result.message}`);
        } else if (parsed.result?.content) {
          console.log('✅ 获得最终结果');
          finalResult = parsed;
        }
      } catch (e) {
        console.log(`⚠️ 行 ${i + 1} 解析失败:`, line);
      }
    }

    // 处理最终结果
    if (finalResult && finalResult.result?.content) {
      console.log('\n🎉 工具调用成功!');
      console.log('📊 解析结果内容:');
      
      finalResult.result.content.forEach((item, index) => {
        console.log(`\n内容 ${index + 1} (${item.type}):`);
        if (item.type === 'text') {
          try {
            const responseData = JSON.parse(item.text);
            console.log('✅ 响应状态:', responseData.status);
            console.log('✅ 请求成功:', responseData.success);
            
            if (responseData.response?.data) {
              console.log('\n📋 Agent 数据:');
              responseData.response.data.forEach((agent, idx) => {
                console.log(`\n  Agent ${idx + 1}:`);
                console.log(`    🆔 ID: ${agent.agentId}`);
                console.log(`    🤖 App ID: ${agent.appId}`);
                console.log(`    🔐 App Secret: ${agent.appSecret}`);
                console.log(`    🔑 API Key: ${agent.agentApiKey}`);
                console.log(`    📅 创建: ${new Date(agent.createTime).toLocaleString()}`);
                console.log(`    🔄 更新: ${new Date(agent.updateTime).toLocaleString()}`);
              });
            }
          } catch (e) {
            console.log('原始文本内容:', item.text);
          }
        }
      });
    } else {
      console.error('❌ 未获得有效结果');
      if (finalResult?.error) {
        console.error('错误信息:', finalResult.error);
      }
    }

  } catch (error) {
    console.error('❌ 测试失败:', error.message);
    console.error('错误堆栈:', error.stack);
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway HTTP Stream 集成测试 (修复版)');
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