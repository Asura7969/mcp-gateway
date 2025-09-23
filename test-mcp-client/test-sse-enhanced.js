#!/usr/bin/env node

/**
 * MCP Gateway SSE 增强测试 - 双向通信版本
 * 使用 EventSource 接收消息 + fetch 发送请求的混合模式
 */

import fetch from 'node-fetch';
import { EventSource } from 'eventsource';

// 配置
const GATEWAY_BASE_URL = 'http://localhost:3000';
const ENDPOINT_ID = 'b0778a81-fba1-4d7b-9539-6d065eae6e22'; // agent-bot endpoint
const AGENT_ID = '98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432';

class McpSseClient {
  constructor(baseUrl, endpointId) {
    this.baseUrl = baseUrl;
    this.endpointId = endpointId;
    this.sseUrl = `${baseUrl}/mcp/${endpointId}/sse`;
    this.requestUrl = `${baseUrl}/mcp/${endpointId}/sse`;
    this.eventSource = null;
    this.requestId = 1;
    this.isReady = false;
    this.serverInfo = null;
    this.tools = [];
  }

  async connect() {
    return new Promise((resolve, reject) => {
      console.log(`🔌 连接到SSE流: ${this.sseUrl}`);
      
      this.eventSource = new EventSource(this.sseUrl);
      
      this.eventSource.onopen = () => {
        console.log('✅ SSE连接已建立');
      };

      this.eventSource.onmessage = (event) => {
        this.handleSseMessage('message', event.data);
      };

      this.eventSource.addEventListener('message', (event) => {
        this.handleSseMessage('message', event.data);
      });

      this.eventSource.addEventListener('ready', (event) => {
        console.log('🟢 服务器就绪:', event.data);
        this.isReady = true;
        resolve();
      });

      this.eventSource.onerror = (error) => {
        console.error('❌ SSE连接错误:', error);
        reject(error);
      };

      // 超时处理
      setTimeout(() => {
        if (!this.isReady) {
          reject(new Error('连接超时'));
        }
      }, 10000);
    });
  }

  handleSseMessage(eventType, data) {
    try {
      const message = JSON.parse(data);
      
      if (message.jsonrpc === '2.0') {
        // 这是MCP消息
        if (message.id === 'server_init') {
          console.log('🖥️  收到服务器初始化信息');
          this.serverInfo = message.result.serverInfo;
          console.log(`   服务器: ${this.serverInfo.name} v${this.serverInfo.version}`);
          console.log(`   协议版本: ${message.result.protocolVersion}`);
        } else if (message.id === 'tools_list') {
          console.log('🔧 收到工具列表');
          this.tools = message.result.tools;
          console.log(`   工具数量: ${this.tools.length}`);
          this.tools.forEach((tool, index) => {
            console.log(`   ${index + 1}. ${tool.name} - ${tool.description}`);
          });
        }
      } else {
        // 非MCP消息
        console.log('📨 收到其他消息:', data);
      }
    } catch (e) {
      console.log('📨 收到原始消息:', data);
    }
  }

  async sendRequest(method, params = {}) {
    if (!this.isReady) {
      throw new Error('服务器尚未就绪');
    }

    const request = {
      jsonrpc: '2.0',
      id: this.requestId++,
      method,
      params
    };

    console.log(`📤 发送${method}请求:`, JSON.stringify(request, null, 2));

    const response = await fetch(this.requestUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();
    console.log(`📥 收到${method}响应:`, JSON.stringify(result, null, 2));
    return result;
  }

  async callTool(toolName, args) {
    return await this.sendRequest('tools/call', {
      name: toolName,
      arguments: args
    });
  }

  async listTools() {
    return await this.sendRequest('tools/list');
  }

  close() {
    if (this.eventSource) {
      this.eventSource.close();
      console.log('🔌 SSE连接已关闭');
    }
  }
}

async function testEnhancedSSE() {
  const client = new McpSseClient(GATEWAY_BASE_URL, ENDPOINT_ID);
  
  try {
    // 1. 连接到SSE流
    console.log('🚀 开始增强SSE测试...');
    await client.connect();
    
    console.log('\n⏳ 等待2秒让服务器完全就绪...');
    await new Promise(resolve => setTimeout(resolve, 2000));

    // 2. 主动请求工具列表（测试双向通信）
    console.log('\n🔧 主动请求工具列表...');
    const toolsResponse = await client.listTools();
    
    if (toolsResponse.result && toolsResponse.result.tools) {
      console.log('✅ 工具列表请求成功');
      console.log(`📊 获得 ${toolsResponse.result.tools.length} 个工具`);
    }

    // 3. 查找并调用findByAgentId工具
    const findAgentTool = client.tools.find(tool => 
      tool.name.includes('findByAgentId') || tool.name.includes('get_bot-agent_findByAgentId')
    );

    if (!findAgentTool) {
      console.log('❌ 未找到findByAgentId工具');
      return;
    }

    console.log(`\n🎯 找到目标工具: ${findAgentTool.name}`);
    console.log('🔧 调用工具获取Agent信息...');

    const toolResult = await client.callTool(findAgentTool.name, {
      query: {
        agentId: AGENT_ID
      }
    });

    // 4. 解析工具调用结果
    if (toolResult.result && toolResult.result.content) {
      console.log('\n🎉 工具调用成功！');
      
      toolResult.result.content.forEach((item, index) => {
        console.log(`\n📋 内容 ${index + 1} (${item.type}):`);
        if (item.type === 'text') {
          try {
            const responseData = JSON.parse(item.text);
            console.log('✅ HTTP状态:', responseData.status);
            console.log('✅ 请求成功:', responseData.success);
            
            if (responseData.response?.data && responseData.response.data.length > 0) {
              console.log('\n🎯 获取到的Agent数据:');
              responseData.response.data.forEach((agent, idx) => {
                console.log(`\n  🤖 Agent ${idx + 1}:`);
                console.log(`    🆔 Agent ID: ${agent.agentId}`);
                console.log(`    📱 App ID: ${agent.appId}`);
                console.log(`    🔐 App Secret: ${agent.appSecret}`);
                console.log(`    🔑 API Key: ${agent.agentApiKey}`);
                console.log(`    📅 创建时间: ${new Date(agent.createTime).toLocaleString()}`);
                console.log(`    🔄 更新时间: ${new Date(agent.updateTime).toLocaleString()}`);
              });
              
              console.log('\n🎉 SSE双向通信测试完全成功！');
              console.log('✅ 成功通过SSE协议获取到Agent数据');
              console.log('✅ SSE协议现在支持完整的MCP双向通信');
            }
          } catch (e) {
            console.log('🔍 原始响应内容:', item.text);
          }
        }
      });
    } else if (toolResult.error) {
      console.error('❌ 工具调用失败:', toolResult.error);
    }

  } catch (error) {
    console.error('❌ 测试失败:', error.message);
    console.error('详细错误:', error);
  } finally {
    // 清理连接
    client.close();
  }
}

// 主函数
async function main() {
  console.log('🧪 MCP Gateway SSE 增强测试 (双向通信)');
  console.log('═'.repeat(60));
  
  try {
    await testEnhancedSSE();
  } catch (error) {
    console.error('主程序执行失败:', error);
    process.exit(1);
  }
  
  console.log('\n🏁 测试完成');
}

// 运行测试
main().catch(console.error);