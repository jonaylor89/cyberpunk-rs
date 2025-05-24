#!/bin/bash

echo "Setting up Cyberpunk MCP Server..."

# Install Node.js dependencies
cd mcp-server
npm install

echo "MCP Server setup complete!"
echo ""
echo "Usage:"
echo "1. Start your cyberpunk-rs server: cargo run"
echo "2. In another terminal, test the MCP server: node index.js"
echo ""
echo "To connect to Claude Desktop, add this to your Claude config:"
echo '{'
echo '  "mcpServers": {'
echo '    "cyberpunk-audio": {'
echo '      "command": "node",'
echo '      "args": ["'$(pwd)'/index.js"],'
echo '      "env": {'
echo '        "CYBERPUNK_SERVER_URL": "http://localhost:8080"'
echo '      }'
echo '    }'
echo '  }'
echo '}'