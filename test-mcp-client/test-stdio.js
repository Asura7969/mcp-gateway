#!/usr/bin/env node

/**
 * MCP Gateway Stdio Client Test
 * 测试通过 stdio 协议连接到 MCP Gateway 并调用 agent-bot 服务的接口
 */

import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { spawn } from 'child_process';

// 配置常量
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint ID
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432'; // 要测试的 agentId

async function testMcpStdioConnection() {
  console.log('🚀 开始测试 MCP Gateway Stdio 连接...');
  console.log(`📡 连接地址: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/stdio`);
  console.log(`🎯 测试接口: /bot-agent/findByAgentId`);
  console.log(`📝 AgentId: ${AGENT_ID}`);
  console.log('─'.repeat(60));

  let client;
  let childProcess;
  
  try {
    // 创建 stdio 子进程
    // 这里使用 curl 作为 stdio 传输的实现
    childProcess = spawn('curl', [
      '-X', 'POST',
      '-H', 'Content-Type: application/json',
      '-H', 'Accept: application/json',
      '--data-binary', '@-',
      `${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/stdio`
    ]);

    // 创建 stdio 传输
    const transport = new StdioClientTransport({
      readable: childProcess.stdout,
      writable: childProcess.stdin
    });

    // 创建 MCP 客户端
    client = new Client(
      {
        name: 'mcp-gateway-stdio-test',
        version: '1.0.0',
      },
      {
        capabilities: {}
      }
    );

    // 连接到服务器
    console.log('🔌 正在通过 stdio 连接到 MCP Gateway...');
    await client.connect(transport);
    console.log('✅ 成功通过 stdio 连接到 MCP Gateway');

    // 获取可用工具列表
    console.log('\n📋 获取可用工具列表...');
    const toolsResponse = await client.listTools();
    console.log('可用工具数量:', toolsResponse.tools.length);
    
    // 显示所有可用工具
    toolsResponse.tools.forEach((tool, index) => {
      console.log(`${index + 1}. ${tool.name}`);
      console.log(`   描述: ${tool.description || '无描述'}`);
      console.log('');
    });

    // 查找目标工具
    const targetTool = toolsResponse.tools.find(tool => 
      tool.name.includes('findByAgentId') || 
      tool.name.includes('bot-agent') ||
      tool.name.includes('get_bot_agent_findbyagentid')
    );

    if (!targetTool) {
      console.log('❌ 未找到 findByAgentId 相关的工具');
      return;
    }

    console.log(`🎯 找到目标工具: ${targetTool.name}`);

    // 调用工具
    console.log('\n🔧 调用工具获取 agent 信息...');
    const callResult = await client.callTool({
      name: targetTool.name,
      arguments: {
        agentId: AGENT_ID
      }
    });

    console.log('✅ 工具调用成功!');
    console.log('📊 返回结果:');
    console.log(JSON.stringify(callResult, null, 2));

  } catch (error) {
    console.error('❌ 测试失败:', error);
    console.error('错误详情:', error.message);
  } finally {
    // 关闭连接
    if (client) {
      try {
        await client.close();
        console.log('\n🔌 已关闭 MCP 连接');
      } catch (closeError) {
        console.error('关闭连接时出错:', closeError);
      }
    }
    
    // 关闭子进程
    if (childProcess) {
      childProcess.kill();
    }
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway Stdio 集成测试');
  console.log('═'.repeat(60));
  
  try {
    await testMcpStdioConnection();
  } catch (error) {
    console.error('主程序执行失败:', error);
    process.exit(1);
  }
  
  console.log('\n🏁 测试完成');
}

// 运行测试
main().catch(console.error);