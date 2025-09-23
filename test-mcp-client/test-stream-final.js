#!/usr/bin/env node

/**
 * MCP Gateway HTTP Stream 最终测试版本
 * 处理连续JSON响应格式
 */

import fetch from 'node-fetch';

// 配置
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432';

function parseStreamResponse(text) {
  const responses = [];
  let currentPos = 0;
  
  while (currentPos < text.length) {
    try {
      // 尝试从当前位置解析JSON
      const remaining = text.slice(currentPos);
      let braceCount = 0;
      let inString = false;
      let escapeNext = false;
      let jsonEnd = -1;
      
      for (let i = 0; i < remaining.length; i++) {
        const char = remaining[i];
        
        if (escapeNext) {
          escapeNext = false;
          continue;
        }
        
        if (char === '\\') {
          escapeNext = true;
          continue;
        }
        
        if (char === '"') {
          inString = !inString;
          continue;
        }
        
        if (!inString) {
          if (char === '{') {
            braceCount++;
          } else if (char === '}') {
            braceCount--;
            if (braceCount === 0) {
              jsonEnd = i;
              break;
            }
          }
        }
      }
      
      if (jsonEnd !== -1) {
        const jsonStr = remaining.slice(0, jsonEnd + 1);
        const parsed = JSON.parse(jsonStr);
        responses.push(parsed);
        currentPos += jsonEnd + 1;
      } else {
        break;
      }
    } catch (e) {
      // 如果解析失败，尝试跳过一个字符
      currentPos++;
    }
  }
  
  return responses;
}

async function testHttpStream() {
  try {
    console.log('🚀 开始测试 MCP Gateway HTTP Stream...');
    console.log(`📡 连接地址: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`);
    console.log(`🎯 测试接口: /bot-agent/findByAgentId`);
    console.log(`📝 AgentId: ${AGENT_ID}`);
    console.log('─'.repeat(60));

    // 1. 调用目标工具
    console.log('\n🔧 直接调用 findByAgentId 工具...');
    const toolCallRequest = {
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: {
        name: 'get_bot-agent_findByAgentId_api',
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
    console.log('📊 原始响应长度:', callText.length, '字符');
    console.log('📊 原始响应预览:', callText.slice(0, 200) + '...');

    // 解析流式响应
    console.log('\n🔍 解析流式响应...');
    const responses = parseStreamResponse(callText);
    console.log(`✅ 成功解析 ${responses.length} 个响应对象`);

    // 处理每个响应
    let finalResult = null;
    responses.forEach((response, index) => {
      console.log(`\n📋 响应 ${index + 1}:`);
      console.log(`  ID: ${response.id}`);
      console.log(`  类型: ${response.result?.type || 'final result'}`);
      
      if (response.result?.type === 'progress') {
        console.log(`  ⏳ 进度消息: ${response.result.message}`);
      } else if (response.result?.content) {
        console.log('  ✅ 最终结果');
        finalResult = response;
      }
    });

    // 处理最终结果
    if (finalResult && finalResult.result?.content) {
      console.log('\n🎉 工具调用成功!');
      console.log('📊 处理结果内容:');
      
      finalResult.result.content.forEach((item, index) => {
        console.log(`\n内容 ${index + 1} (${item.type}):`);
        if (item.type === 'text') {
          try {
            const responseData = JSON.parse(item.text);
            console.log('✅ HTTP状态:', responseData.status);
            console.log('✅ 请求成功:', responseData.success);
            
            if (responseData.response?.data && responseData.response.data.length > 0) {
              console.log('\n🎯 找到的Agent数据:');
              responseData.response.data.forEach((agent, idx) => {
                console.log(`\n  🤖 Agent ${idx + 1}:`);
                console.log(`    🆔 Agent ID: ${agent.agentId}`);
                console.log(`    📱 App ID: ${agent.appId}`);
                console.log(`    🔐 App Secret: ${agent.appSecret}`);
                console.log(`    🔑 API Key: ${agent.agentApiKey}`);
                console.log(`    📅 创建时间: ${new Date(agent.createTime).toLocaleString()}`);
                console.log(`    🔄 更新时间: ${new Date(agent.updateTime).toLocaleString()}`);
              });
              
              console.log('\n✅ Streamable协议测试完全成功!');
              console.log('✅ 成功通过流式协议获取到Agent数据');
            } else {
              console.log('⚠️ 响应中没有Agent数据');
            }
          } catch (e) {
            console.log('原始文本内容:', item.text.slice(0, 200) + '...');
          }
        }
      });
    } else {
      console.error('❌ 未获得有效的最终结果');
    }

  } catch (error) {
    console.error('❌ 测试失败:', error.message);
    if (error.cause) {
      console.error('原因:', error.cause);
    }
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway HTTP Stream 最终测试');
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