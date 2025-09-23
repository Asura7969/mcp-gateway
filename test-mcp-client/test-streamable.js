#!/usr/bin/env node

/**
 * MCP Gateway Streamable HTTP 测试
 * 测试Streamable HTTP协议与官方MCP Inspector的兼容性
 */

import axios from 'axios';

// 配置
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = '0b88fc39-16c8-4238-bee8-11503522ba95'; // 替换为实际的endpoint ID

async function testStreamable() {
  try {
    console.log('🚀 开始测试 MCP Gateway Streamable HTTP连接...');
    console.log(`📡 Streamable URL: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`);
    console.log('─'.repeat(60));

    // 1. 发送initialize请求
    console.log('\n📋 发送 initialize 请求...');
    const initResponse = await axios.post(
      `${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`,
      {
        jsonrpc: '2.0',
        id: 1,
        method: 'initialize',
        params: {
          protocolVersion: '2024-11-05',
          capabilities: {
            tools: {},
            resources: {},
            prompts: {}
          },
          clientInfo: {
            name: 'mcp-streamable-test-client',
            version: '1.0.0'
          }
        }
      },
      {
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );
    
    console.log('✅ Initialize响应:', JSON.stringify(initResponse.data, null, 2));

    // 2. 发送tools/list请求
    console.log('\n📋 发送 tools/list 请求...');
    const toolsResponse = await axios.post(
      `${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`,
      {
        jsonrpc: '2.0',
        id: 2,
        method: 'tools/list'
      },
      {
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );
    
    console.log('✅ Tools列表响应:', JSON.stringify(toolsResponse.data, null, 2));
    
    // 3. 发送resources/list请求
    console.log('\n📋 发送 resources/list 请求...');
    const resourcesResponse = await axios.post(
      `${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`,
      {
        jsonrpc: '2.0',
        id: 3,
        method: 'resources/list'
      },
      {
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );
    
    console.log('✅ Resources列表响应:', JSON.stringify(resourcesResponse.data, null, 2));

    // 4. 发送prompts/list请求
    console.log('\n📋 发送 prompts/list 请求...');
    const promptsResponse = await axios.post(
      `${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/streamable`,
      {
        jsonrpc: '2.0',
        id: 4,
        method: 'prompts/list'
      },
      {
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );
    
    console.log('✅ Prompts列表响应:', JSON.stringify(promptsResponse.data, null, 2));

    console.log('\n✅ Streamable HTTP测试完成!');
    
  } catch (error) {
    console.error('❌ Streamable HTTP测试失败:', error.message);
    if (error.response) {
      console.error('响应状态:', error.response.status);
      console.error('响应数据:', JSON.stringify(error.response.data, null, 2));
    }
    if (error.cause) {
      console.error('错误原因:', error.cause);
    }
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway Streamable HTTP 测试');
  console.log('═'.repeat(60));
  
  try {
    await testStreamable();
  } catch (error) {
    console.error('主程序执行失败:', error);
    process.exit(1);
  }
  
  console.log('\n🏁 测试完成');
}

// 运行测试
main().catch(console.error);