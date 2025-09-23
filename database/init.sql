-- Create database if not exists
CREATE DATABASE IF NOT EXISTS mcp_gateway;
USE mcp_gateway;

-- Grant privileges to the user
GRANT ALL PRIVILEGES ON mcp_gateway.* TO 'mcpuser'@'%';
FLUSH PRIVILEGES;