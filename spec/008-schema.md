# 优化

```json
{
  "components": {
    "schemas": {
      "BotAgentDto": {
        "type": "object",
        "properties": {
          "agentApiKey": {
            "type": "string",
            "description": "API密钥"
          },
          "agentId": {
            "type": "string",
            "description": "Agent ID"
          },
          "appEncryptKey": {
            "type": "string",
            "description": "应用加密密钥"
          },
          "appId": {
            "type": "string",
            "description": "应用ID"
          },
          "appSecret": {
            "type": "string",
            "description": "应用密钥"
          },
          "appVerificationToken": {
            "type": "string",
            "description": "应用验证令牌"
          },
          "createTime": {
            "type": "string",
            "description": "创建时间"
          },
          "updateTime": {
            "type": "string",
            "description": "更新时间"
          }
        },
        "required": [
          "appId",
          "appSecret",
          "agentId",
          "agentApiKey"
        ]
      },
      "ResultBoolean": {
        "type": "object",
        "properties": {
          "code": {
            "type": "integer",
            "description": "状态码"
          },
          "data": {
            "type": "boolean",
            "description": "数据"
          },
          "msg": {
            "type": "string",
            "description": "消息"
          },
          "success": {
            "type": "boolean",
            "description": "是否成功"
          },
          "timestamp": {
            "type": "integer",
            "format": "int64",
            "description": "时间戳"
          }
        }
      }
    }
  },
  "info": {
    "description": "机器人接口",
    "title": "agent-bot",
    "version": "1.0.0"
  },
  "openapi": "3.1.0",
  "paths": {
    "/bot-agent/findByAgentId": {
      "get": {
        "description": "根据AgentId查询机器人信息",
        "operationId": "findByAgentId",
        "parameters": [
          {
            "description": "agentId",
            "in": "query",
            "name": "agentId",
            "required": true,
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ResultBoolean"
                }
              }
            }
          }
        },
        "summary": "机器人查询接口",
        "tags": []
      }
    },
    "/bot-agent/save": {
      "post": {
        "description": "保存机器人与agent的关系",
        "operationId": "saveBotAgent",
        "parameters": [],
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/BotAgentDto"
              }
            }
          },
          "description": "机器人Agent信息",
          "required": true
        },
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ResultBoolean"
                }
              }
            }
          }
        },
        "summary": "保存机器人-agent关系",
        "tags": []
      }
    }
  },
  "servers": [
    {
      "url": "http://ai-service.dev.starcharge.cloud"
    }
  ]
}
```

帮我优化`McpTool`对象和`generate_mcp_tools`方法，使最后生成的`McpTool`对象包含如下字段：

name：字符串
title：字符串
description：字符串
inputSchema：入参json对象
outputSchema：出参json对象，有响应格式就解析，没有则无

inputSchema json对象需包含如下字段：
type：字符串，一般值为'object'
properties：json对象
required：数组，必填字段

properties对象需包含如下字段：
```json
{
    "参数名": {"type": "参数类型", "description": "参数描述"}
}
```

outputSchema输出格式与inputSchema一致