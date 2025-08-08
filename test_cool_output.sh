#!/bin/bash

# Test script to demonstrate the cool new reasoning output

echo "═══════════════════════════════════════"
echo "⚡ LUX MCP - COGNITIVE ENGINE TEST ⚡"
echo "═══════════════════════════════════════"
echo ""

# Colors for output
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

# Start server in background
echo -e "${CYAN}⚡ Initializing reasoning engine...${NC}"
RUST_LOG=info ./target/release/lux-mcp 2>test_output.log &
SERVER_PID=$!
sleep 2

# Test 1: Biased Reasoning with cool output
echo -e "\n${YELLOW}🔮 TEST 1: COGNITIVE FRAME PROCESSING${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"

cat << 'EOF' | nc -w 5 localhost 3333 2>/dev/null | python3 -m json.tool 2>/dev/null | grep -A 20 "content" || echo "Connection failed"
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "biased_reasoning",
    "arguments": {
      "query": "What are the best practices for building scalable microservices?"
    }
  }
}
EOF

sleep 2

# Test 2: Traced Reasoning
echo -e "\n${YELLOW}⚡ TEST 2: REASONING CHAIN ACTIVATION${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"

cat << 'EOF' | nc -w 5 localhost 3333 2>/dev/null | python3 -m json.tool 2>/dev/null | grep -A 20 "thought_content" || echo "Connection failed"
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "thought": "How can we optimize database performance in distributed systems?",
      "thought_number": 1,
      "total_thoughts": 3,
      "next_thought_needed": true
    }
  }
}
EOF

sleep 1

# Test 3: Chat with model indicator
echo -e "\n${YELLOW}💫 TEST 3: CONVERSATIONAL ENGINE${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"

cat << 'EOF' | nc -w 5 localhost 3333 2>/dev/null | python3 -m json.tool 2>/dev/null | grep -A 10 "response" || echo "Connection failed"
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Explain quantum computing in simple terms",
      "model": "o3-mini"
    }
  }
}
EOF

# Show some logs
echo -e "\n${GREEN}📊 ENGINE DIAGNOSTICS:${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"
tail -20 test_output.log | grep -E "⚡|🔮|🎯|Cognitive|Frame|Engine" || echo "No matching logs"

# Cleanup
echo -e "\n${CYAN}⚡ Shutting down reasoning engine...${NC}"
kill $SERVER_PID 2>/dev/null
rm -f test_output.log

echo -e "\n${GREEN}✅ TEST COMPLETE${NC}"
echo -e "${CYAN}═══════════════════════════════════════${NC}"
echo ""
echo "The reasoning engine now uses:"
echo "  ⚡ Cognitive Frames instead of thoughts"
echo "  🔮 Reasoning Engine instead of brain"
echo "  💫 Processing indicators instead of thinking"
echo "  🎯 Target-based conclusions"
echo ""
echo "Check the output above to see the new cool formatting!"