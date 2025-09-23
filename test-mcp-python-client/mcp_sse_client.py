#!/usr/bin/env python3
"""
MCP SSE Client Test
测试连接到MCP Gateway的SSE传输协议
"""

import asyncio
import json
import requests
import sseclient
from typing import Optional, Dict, Any
from urllib.parse import urljoin

class MCPSSEClient:
    def __init__(self, base_url: str, endpoint_id: str):
        self.base_url = base_url
        self.endpoint_id = endpoint_id
        self.sse_url = f"{base_url}/mcp/{endpoint_id}/sse"
        self.session_id: Optional[str] = None
        self.message_url: Optional[str] = None
        
    def connect_sse(self):
        """建立SSE连接并获取消息端点"""
        print(f"连接到SSE端点: {self.sse_url}")
        
        try:
            response = requests.get(self.sse_url, stream=True, headers={
                'Accept': 'text/event-stream',
                'Cache-Control': 'no-cache'
            })
            response.raise_for_status()
            
            client = sseclient.SSEClient(response)
            
            for event in client.events():
                print(f"收到SSE事件: {event.event}, 数据: {event.data}")
                
                if event.event == 'endpoint':
                    # 解析endpoint事件获取消息URL
                    endpoint_data = json.loads(event.data)
                    relative_uri = endpoint_data.get('uri')
                    if relative_uri:
                        # 提取session_id
                        if 'session_id=' in relative_uri:
                            self.session_id = relative_uri.split('session_id=')[1]
                            self.message_url = urljoin(self.base_url, relative_uri)
                            print(f"获取到session_id: {self.session_id}")
                            print(f"消息端点URL: {self.message_url}")
                            return True
                        
                # 只处理第一个endpoint事件
                break
                
        except Exception as e:
            print(f"SSE连接错误: {e}")
            return False
            
        return False
    
    def send_mcp_request(self, method: str, params: Optional[Dict[str, Any]] = None, request_id: int = 1) -> Optional[Dict[str, Any]]:
        """发送MCP请求"""
        if not self.message_url:
            print("错误: 未建立SSE连接或未获取到消息端点")
            return None
            
        mcp_request = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params or {}
        }
        
        print(f"发送MCP请求到 {self.message_url}:")
        print(f"请求内容: {json.dumps(mcp_request, indent=2)}")
        
        try:
            response = requests.post(
                self.message_url,
                json=mcp_request,
                headers={'Content-Type': 'application/json'}
            )
            response.raise_for_status()
            
            if response.content:
                result = response.json()
                print(f"响应状态: {response.status_code}")
                print(f"响应内容: {json.dumps(result, indent=2, ensure_ascii=False)}")
                return result
            else:
                print(f"响应状态: {response.status_code} (无内容)")
                return {"status": "success", "status_code": response.status_code}
                
        except requests.exceptions.RequestException as e:
            print(f"请求错误: {e}")
            return None
        except json.JSONDecodeError as e:
            print(f"JSON解析错误: {e}")
            print(f"原始响应: {response.text}")
            return None
    
    def test_initialize(self):
        """测试初始化"""
        print("\n=== 测试初始化 ===")
        return self.send_mcp_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "mcp-sse-test-client",
                "version": "1.0.0"
            }
        })
    
    def test_list_tools(self):
        """测试获取工具列表"""
        print("\n=== 测试获取工具列表 ===")
        return self.send_mcp_request("tools/list")
    
    def test_call_tool(self, tool_name: str, arguments: Optional[Dict[str, Any]] = None):
        """测试调用工具"""
        print(f"\n=== 测试调用工具: {tool_name} ===")
        return self.send_mcp_request("tools/call", {
            "name": tool_name,
            "arguments": arguments or {}
        })
    
    def test_call_tool_with_agent_id(self, tool_name: str, agent_id: str):
        """测试调用带agentId参数的工具"""
        print(f"\n=== 测试调用工具: {tool_name} (agentId: {agent_id}) ===")
        return self.send_mcp_request("tools/call", {
            "name": tool_name,
            "arguments": {"agentId": agent_id}
        })

def main():
    """主测试函数"""
    # 配置
    BASE_URL = "http://localhost:3000"
    ENDPOINT_ID = "2764a1cc-4513-4726-ae88-05d33d164493"
    
    print("MCP SSE客户端集成测试")
    print(f"服务器地址: {BASE_URL}")
    print(f"端点ID: {ENDPOINT_ID}")
    print("=" * 50)
    
    # 创建客户端
    client = MCPSSEClient(BASE_URL, ENDPOINT_ID)
    
    # 建立SSE连接
    if not client.connect_sse():
        print("❌ SSE连接失败")
        return
    
    print("✅ SSE连接成功")
    
    # 测试初始化
    init_result = client.test_initialize()
    if init_result:
        print("✅ 初始化成功")
    else:
        print("❌ 初始化失败")
    
    # 测试获取工具列表
    tools_result = client.test_list_tools()
    if tools_result:
        print("✅ 获取工具列表成功")
        
        # 如果有工具，测试调用第一个工具
        if 'result' in tools_result and 'tools' in tools_result['result']:
            tools = tools_result['result']['tools']
            if tools:
                # 测试findByAgentId工具，使用提供的agentId参数
                find_tool = None
                for tool in tools:
                    if 'findByAgentId' in tool.get('name', ''):
                        find_tool = tool
                        break
                
                if find_tool:
                    tool_name = find_tool.get('name')
                    agent_id = "98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432"
                    call_result = client.test_call_tool_with_agent_id(tool_name, agent_id)
                    if call_result:
                        print(f"✅ 工具调用成功: {tool_name}")
                    else:
                        print(f"❌ 工具调用失败: {tool_name}")
                
                # 测试POST工具（保存机器人-agent关系）
                save_tool = None
                for tool in tools:
                    if 'save' in tool.get('name', ''):
                        save_tool = tool
                        break
                
                if save_tool:
                    tool_name = save_tool.get('name')
                    # 测试body参数传递
                    test_data = {
                        "agentId": "98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432",
                        "botId": "test-bot-123",
                        "appId": "cli_a5f8e3d9b7c401ab"
                    }
                    call_result = client.test_call_tool(tool_name, {"body": test_data})
                    if call_result:
                        print(f"✅ POST工具调用成功: {tool_name}")
                    else:
                        print(f"❌ POST工具调用失败: {tool_name}")
                else:
                    # 如果没有findByAgentId工具，测试第一个工具
                    first_tool = tools[0]
                    tool_name = first_tool.get('name')
                    if tool_name:
                        call_result = client.test_call_tool(tool_name)
                        if call_result:
                            print(f"✅ 工具调用成功: {tool_name}")
                        else:
                            print(f"❌ 工具调用失败: {tool_name}")
    else:
        print("❌ 获取工具列表失败")
    
    print("\n=== 测试完成 ===")

if __name__ == "__main__":
    main()