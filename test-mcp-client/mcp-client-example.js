#!/usr/bin/env node

/**
 * MCP SDK 真实使用示例
 * 演示如何在客户端应用中集成MCP Gateway
 */

import fetch from 'node-fetch';

// MCP 客户端封装类
class McpGatewayClient {
  constructor(gatewayUrl, endpointId) {
    this.gatewayUrl = gatewayUrl;
    this.endpointId = endpointId;
    this.baseUrl = `${gatewayUrl}/mcp/${endpointId}/stdio`;
    this.requestId = 1;
  }

  async request(method, params = {}) {
    const request = {
      jsonrpc: '2.0',
      id: this.requestId++,
      method,
      params
    };

    const response = await fetch(this.baseUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });

    const data = await response.json();
    
    if (data.error) {
      throw new Error(`MCP Error: ${JSON.stringify(data.error)}`);
    }

    return data.result;
  }

  async listTools() {
    return await this.request('tools/list');
  }

  async callTool(name, args) {
    return await this.request('tools/call', { name, arguments: args });
  }
}

// Agent Bot 服务客户端
class AgentBotClient {
  constructor(mcpClient) {
    this.mcp = mcpClient;
    this.tools = null;
  }

  async initialize() {
    const toolsResult = await this.mcp.listTools();
    this.tools = toolsResult.tools;
    console.log(`🔧 已加载 ${this.tools.length} 个工具`);
  }

  async findAgentById(agentId) {
    const tool = this.tools.find(t => t.name.includes('findByAgentId'));
    if (!tool) {
      throw new Error('findByAgentId 工具未找到');
    }

    const result = await this.mcp.callTool(tool.name, {
      query: { agentId }
    });

    // 解析返回的文本内容
    const textContent = result.content.find(c => c.type === 'text');
    if (textContent) {
      const response = JSON.parse(textContent.text);
      if (response.success && response.response && response.response.data) {
        return response.response.data;
      }
    }

    throw new Error('获取Agent数据失败');
  }

  async saveAgent(agentData) {
    const tool = this.tools.find(t => t.name.includes('save'));
    if (!tool) {
      throw new Error('save 工具未找到');
    }

    const result = await this.mcp.callTool(tool.name, {
      body: agentData
    });

    return result;
  }
}

// 使用示例
async function main() {
  console.log('🚀 启动 MCP Gateway 客户端示例');
  
  // 创建MCP客户端
  const mcpClient = new McpGatewayClient(
    'http://localhost:3000',
    'b0778a81-fba1-4d7b-9539-6d065eae6e22'
  );

  // 创建Agent Bot服务客户端
  const agentBot = new AgentBotClient(mcpClient);
  
  try {
    // 初始化
    await agentBot.initialize();
    
    // 查询Agent
    console.log('\n🔍 查询Agent信息...');
    const agents = await agentBot.findAgentById('98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432');
    
    console.log('✅ 查询成功!');
    console.log('📋 Agent信息:');
    agents.forEach((agent, index) => {
      console.log(`\n  Agent ${index + 1}:`);
      console.log(`    🆔 ID: ${agent.agentId}`);
      console.log(`    🤖 App ID: ${agent.appId}`);
      console.log(`    🔐 App Secret: ${agent.appSecret}`);
      console.log(`    🔑 API Key: ${agent.agentApiKey}`);
      console.log(`    📅 创建: ${new Date(agent.createTime).toLocaleString()}`);
      console.log(`    🔄 更新: ${new Date(agent.updateTime).toLocaleString()}`);
    });

    console.log('\n✅ MCP Gateway 集成测试完成!');
    console.log('👍 您可以在您的应用中使用类似的方式集成MCP Gateway');
    
  } catch (error) {
    console.error('❌ 错误:', error.message);
    process.exit(1);
  }
}

main();