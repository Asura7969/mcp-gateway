#!/usr/bin/env node

/**
 * MCP Gateway SSE 测试 - 修复版本
 * 测试Server-Sent Events协议
 */

import { SSEClientTransport } from '@modelcontextprotocol/sdk/client/sse.js';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';

// 配置
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432';

async function testSSE() {
  let client = null;
  
  try {
    console.log('🚀 开始测试 MCP Gateway SSE连接...');
    console.log(`📡 SSE URL: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/sse`);
    console.log(`🎯 测试接口: /bot-agent/findByAgentId`);
    console.log(`📝 AgentId: ${AGENT_ID}`);
    console.log('─'.repeat(60));

    // 创建SSE传输客户端
    console.log('\n🔌 创建SSE客户端...');
    const transport = new SSEClientTransport(`${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/sse`);
    client = new Client({
      name: 'mcp-sse-test-client',
      version: '1.0.0'
    }, {
      capabilities: {}
    });

    // 连接到服务器
    console.log('🔗 正在连接到MCP服务器...');
    await client.connect(transport);
    console.log('✅ 成功连接到MCP服务器');

    // 获取服务器信息
    console.log('\n📋 获取服务器信息...');
    const serverInfo = await client.getServerVersion();
    console.log('🖥️  服务器信息:', JSON.stringify(serverInfo, null, 2));

    // 获取工具列表
    console.log('\n📋 获取可用工具列表...');
    const toolsResponse = await client.listTools();
    console.log('✅ 成功获取工具列表');
    console.log(`📊 可用工具数量: ${toolsResponse.tools.length}`);
    
    // 显示所有可用工具
    toolsResponse.tools.forEach((tool, index) => {
      console.log(`${index + 1}. ${tool.name}`);
      console.log(`   📝 描述: ${tool.description || '无描述'}`);
      console.log(`   📋 参数: ${JSON.stringify(tool.inputSchema?.properties || {}, null, 2)}`);
      console.log('');
    });

    // 查找目标工具
    const targetTool = toolsResponse.tools.find(tool => 
      tool.name.includes('findByAgentId') || 
      tool.name.includes('get_bot-agent_findByAgentId')
    );

    if (!targetTool) {
      console.log('❌ 未找到 findByAgentId 相关的工具');
      console.log('可用工具列表:');
      toolsResponse.tools.forEach(tool => console.log(`  - ${tool.name}`));
      return;
    }

    console.log(`🎯 找到目标工具: ${targetTool.name}`);
    console.log('🔍 工具详情:', JSON.stringify(targetTool, null, 2));

    // 调用工具
    console.log('\n🔧 调用工具获取 agent 信息...');
    const callResult = await client.callTool({
      name: targetTool.name,
      arguments: {
        query: {
          agentId: AGENT_ID
        }
      }
    });

    console.log('✅ 工具调用成功!');
    console.log('📊 返回结果类型:', typeof callResult);
    console.log('📊 返回结果:', JSON.stringify(callResult, null, 2));

    // 解析返回内容
    if (callResult.content && callResult.content.length > 0) {
      console.log('\n🎯 解析返回内容:');
      callResult.content.forEach((item, index) => {
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
              
              console.log('\n✅ SSE协议测试完全成功!');
              console.log('✅ 成功通过SSE协议获取到Agent数据');
            } else {
              console.log('⚠️ 响应中没有Agent数据');
            }
          } catch (e) {
            console.log('🔍 原始文本内容:', item.text);
          }
        }
      });
    }

  } catch (error) {
    console.error('❌ SSE测试失败:', error.message);
    console.error('错误详情:', error);
    if (error.cause) {
      console.error('错误原因:', error.cause);
    }
  } finally {
    // 清理连接
    if (client) {
      try {
        console.log('\n🔌 关闭 MCP 连接...');
        await client.close();
        console.log('✅ 连接已关闭');
      } catch (closeError) {
        console.error('关闭连接时出错:', closeError);
      }
    }
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway SSE 集成测试 (修复版)');
  console.log('═'.repeat(60));
  
  try {
    await testSSE();
  } catch (error) {
    console.error('主程序执行失败:', error);
    process.exit(1);
  }
  
  console.log('\n🏁 测试完成');
}

// 运行测试
main().catch(console.error);