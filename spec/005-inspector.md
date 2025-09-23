# 兼容官方mcp inspector

本项目inspector页未与官方实现一致，导致使用官方inspector工具调试本项目时，出现异常（sse、stdio、streamable）。

官方代码在inspector目录下

## 任务
* 与官方inspector页实现一致（页面布局与功能）
* 支持调试本项目提供的mcp-server（功能）
* 支持调试其他项目提供的mcp-server（功能）
* 支持调试其他项目提供的mcp-server的tools、resources（功能）
* 各协议实现与官方一致
* 工具列表的参数与实际swagger定义一致，调用工具时展示参数
* 工具调用完成后展示响应结果，且优化显示（格式化、高亮）

## 重点
### 协议与响应格式
> 参考
下面是魔搭社区提供的mcp-server，提供了一个maps_weather工具
```
{
  "mcpServers": {
    "amap-maps": {
      "type": "sse",
      "url": "https://mcp.api-inference.modelscope.net/9645547a35d641/sse"
    }
  }
}
```
官方Inspector页面连接上述mcp-server的请求
```shell
curl 'http://localhost:6277/sse?url=https%3A%2F%2Fmcp.api-inference.modelscope.net%2F9645547a35d641%2Fsse&transportType=sse' \
  -H 'Accept: */*' \
  -H 'Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:6274' \
  -H 'Referer: http://localhost:6274/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0' \
  -H 'X-MCP-Proxy-Auth: Bearer 35d9106c12bfaf1c7d27d0a22a2a0d8f4d70148139a846f8a92370fd5272eecf' \
  -H 'sec-ch-ua: "Not;A=Brand";v="99", "Microsoft Edge";v="139", "Chromium";v="139"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"'


curl 'http://localhost:6277/message?sessionId=96936be2-0a36-4aa3-aecd-9bd5f0b4250e' \
  -H 'Accept: */*' \
  -H 'Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:6274' \
  -H 'Referer: http://localhost:6274/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0' \
  -H 'content-type: application/json' \
  -H 'sec-ch-ua: "Not;A=Brand";v="99", "Microsoft Edge";v="139", "Chromium";v="139"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'x-mcp-proxy-auth: Bearer 35d9106c12bfaf1c7d27d0a22a2a0d8f4d70148139a846f8a92370fd5272eecf' \
  --data-raw '{"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{"sampling":{},"elicitation":{},"roots":{"listChanged":true}},"clientInfo":{"name":"mcp-inspector","version":"0.16.6"}},"jsonrpc":"2.0","id":0}'

curl 'http://localhost:6277/message?sessionId=96936be2-0a36-4aa3-aecd-9bd5f0b4250e' \
  -H 'Accept: */*' \
  -H 'Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:6274' \
  -H 'Referer: http://localhost:6274/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0' \
  -H 'content-type: application/json' \
  -H 'mcp-protocol-version: 2025-03-26' \
  -H 'sec-ch-ua: "Not;A=Brand";v="99", "Microsoft Edge";v="139", "Chromium";v="139"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'x-mcp-proxy-auth: Bearer 35d9106c12bfaf1c7d27d0a22a2a0d8f4d70148139a846f8a92370fd5272eecf' \
  --data-raw '{"method":"notifications/initialized","jsonrpc":"2.0"}'
```

请求工具列表
```shell
curl 'http://localhost:6277/message?sessionId=96936be2-0a36-4aa3-aecd-9bd5f0b4250e' \
  -H 'Accept: */*' \
  -H 'Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:6274' \
  -H 'Referer: http://localhost:6274/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0' \
  -H 'content-type: application/json' \
  -H 'mcp-protocol-version: 2025-03-26' \
  -H 'sec-ch-ua: "Not;A=Brand";v="99", "Microsoft Edge";v="139", "Chromium";v="139"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'x-mcp-proxy-auth: Bearer 35d9106c12bfaf1c7d27d0a22a2a0d8f4d70148139a846f8a92370fd5272eecf' \
  --data-raw '{"method":"tools/list","params":{"_meta":{"progressToken":1}},"jsonrpc":"2.0","id":1}'
```


官方Inspector调用maps_weather工具的curl请求如下：
```shell
curl 'http://localhost:6277/message?sessionId=e5f261cf-39cf-458c-9d0b-1a8b769ef031' \
  -H 'Accept: */*' \
  -H 'Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6' \
  -H 'Connection: keep-alive' \
  -H 'Origin: http://localhost:6274' \
  -H 'Referer: http://localhost:6274/' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-site' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0' \
  -H 'content-type: application/json' \
  -H 'mcp-protocol-version: 2025-03-26' \
  -H 'sec-ch-ua: "Not;A=Brand";v="99", "Microsoft Edge";v="139", "Chromium";v="139"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'x-mcp-proxy-auth: Bearer 35d9106c12bfaf1c7d27d0a22a2a0d8f4d70148139a846f8a92370fd5272eecf' \
  --data-raw '{"method":"tools/call","params":{"name":"maps_weather","arguments":{"city":"常州"},"_meta":{"progressToken":1}},"jsonrpc":"2.0","id":1}'
```
本项目提供的mcp-server需与上述调用一致，保证服务端返回的响应格式和协议与官方Inspector页面mcp-client接受的内容一致。

### 测试
测试本项目提供的mcp-server是否与上述的预期结果一致