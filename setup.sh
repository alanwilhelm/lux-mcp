#!/bin/bash

# Lux MCP Setup Script

echo "🔦 Lux MCP Setup"
echo "================"
echo

# Check if .env exists
if [ ! -f .env ]; then
    echo "Creating .env file from template..."
    cp .env.example .env
    echo "✓ Created .env file"
    echo
    echo "⚠️  Please edit .env and add your API keys:"
    echo "   - OPENAI_API_KEY"
    echo "   - OPENROUTER_API_KEY (optional)"
    echo
else
    echo "✓ .env file already exists"
fi

# Build the project
echo
echo "Building Lux MCP..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
else
    echo "✗ Build failed"
    exit 1
fi

# Make scripts executable
chmod +x test_*.sh
echo "✓ Made test scripts executable"

# Test the binary
echo
echo "Testing the server..."
./target/release/lux-mcp --version 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✓ Server is working"
else
    echo "Testing server capabilities..."
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./target/release/lux-mcp | jq '.result.capabilities' 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✓ Server responds to MCP protocol"
    else
        echo "⚠️  Server may need API keys configured"
    fi
fi

# Show Claude Desktop config path
echo
echo "📋 Claude Desktop Configuration"
echo "==============================="
echo
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_PATH="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    CONFIG_PATH="$APPDATA/Claude/claude_desktop_config.json"
else
    CONFIG_PATH="$HOME/.config/Claude/claude_desktop_config.json"
fi

echo "Add this to your Claude Desktop config at:"
echo "$CONFIG_PATH"
echo
echo '{
  "mcpServers": {
    "lux": {
      "command": "'$(pwd)'/target/release/lux-mcp",
      "env": {
        "OPENAI_API_KEY": "your-openai-key",
        "OPENROUTER_API_KEY": "your-openrouter-key",
        "LUX_DEFAULT_CHAT_MODEL": "gpt4.1",
        "LUX_DEFAULT_REASONING_MODEL": "o3",
        "LUX_DEFAULT_BIAS_CHECKER_MODEL": "o4-mini"
      }
    }
  }
}'

echo
echo "✨ Setup complete!"
echo
echo "Next steps:"
echo "1. Add your API keys to .env"
echo "2. Add the configuration above to Claude Desktop"
echo "3. Restart Claude Desktop"
echo "4. Test with: ./test_chat.sh"
echo
echo "For more details, see SETUP_GUIDE.md"