# 007 - RMCP Implementation

## Overview

This document describes the implementation of the Model Context Protocol (MCP) using the Rust Model Context Protocol (RMCP) crate. The implementation provides a dynamic MCP server that can serve endpoints based on Swagger/OpenAPI specifications.

## Implementation Details

### Dynamic RMCP Server

The dynamic RMCP server is implemented in `src/services/dynamic_rmcp_server.rs`. This server dynamically creates MCP tools based on Swagger/OpenAPI specifications and executes HTTP requests to the actual APIs when tools are called.

Key features:
- Dynamic tool generation from Swagger specifications
- HTTP request execution based on tool calls
- Caching of endpoint information for performance
- Support for multiple HTTP methods (GET, POST, PUT, DELETE, PATCH)
- Parameter extraction from Swagger specs (path, query, body parameters)

### RMCP Service

The RMCP service is implemented in `src/services/rmcp_service.rs`. This service provides a bridge between the existing database-based endpoint management and the RMCP-based MCP server implementation.

### Streamable HTTP Protocol Support

The implementation includes full support for the Streamable HTTP protocol, which allows for real-time communication between MCP clients and servers using HTTP with Server-Sent Events (SSE).

Key features of Streamable HTTP implementation:
- Session management with unique session IDs
- Support for GET, POST, and DELETE HTTP methods
- SSE streaming for real-time message delivery
- Request/response correlation using event IDs
- Session resumption capability
- Proper HTTP header handling (Content-Type, Accept, etc.)

### Integration with Existing System

The RMCP implementation is integrated with the existing MCP gateway system through:

1. Database integration for endpoint management
2. Swagger specification parsing for tool generation
3. HTTP client for executing API calls
4. Axum web framework for HTTP transport

## Usage

### Starting the RMCP Server

The RMCP server can be started using the dedicated binary:

```bash
cargo run --bin mcp_rmcp_server
```

This will start the server on port 3001.

### Available Endpoints

The RMCP server provides the following endpoints:

1. `/health` - Health check endpoint
2. `/mcp/{endpoint_id}/stdio` - Stdio transport endpoint
3. `/mcp/{endpoint_id}/sse` - SSE transport endpoint
4. `/mcp/{endpoint_id}/message` - SSE message endpoint
5. `/streamable-http` - Streamable HTTP service endpoint (RMCP native implementation)

### Streamable HTTP Protocol Endpoints

The Streamable HTTP protocol is implemented as a native RMCP transport and is available at `/streamable-http`. It supports:

- POST requests for sending JSON-RPC messages
- GET requests for establishing SSE streams
- DELETE requests for closing sessions
- Automatic session management with session IDs
- Event ID tracking for message correlation

## Implementation Files

1. `src/services/dynamic_rmcp_server.rs` - Main RMCP server implementation
2. `src/services/rmcp_service.rs` - RMCP service bridge
3. `src/bin/mcp_rmcp_server.rs` - Dedicated RMCP server binary
4. `src/main.rs` - Integration with main application
5. `Cargo.toml` - Dependencies and features

## Features

- Dynamic tool generation from Swagger specifications
- HTTP request execution based on tool calls
- Support for multiple HTTP methods
- Parameter extraction from Swagger specs
- Session management for Streamable HTTP
- Real-time communication using SSE
- Integration with existing database and endpoint management
- Support for all RMCP transport protocols (stdio, SSE, Streamable HTTP)

## Testing

Integration tests are available in `tests/rmcp_service_tests.rs` to verify the functionality of the RMCP implementation.

## Future Improvements

- Enhanced error handling and logging
- Performance optimizations for high-load scenarios
- Additional transport protocol support
- Extended tool generation capabilities
- Improved session management for Streamable HTTP