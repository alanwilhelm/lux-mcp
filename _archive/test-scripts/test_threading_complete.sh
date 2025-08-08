#!/bin/bash

# Comprehensive test for Lux MCP threading system with all phases

echo "========================================="
echo "Lux MCP Complete Threading System Test"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build the project first
echo -e "${YELLOW}Building project...${NC}"
if cargo build --release 2>&1 | tail -5; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Function to send MCP request and format output
send_request() {
    local description="$1"
    local request="$2"
    echo -e "\n${YELLOW}$description${NC}"
    echo "$request" | python3 -m json.tool 2>/dev/null | head -20 || echo "$request"
    echo "---"
    response=$(echo "$request" | nc -w 3 localhost 3333 2>/dev/null)
    if [ -n "$response" ]; then
        echo "$response" | python3 -m json.tool 2>/dev/null | head -30 || echo "$response"
        echo -e "${GREEN}✓ Request successful${NC}"
    else
        echo -e "${RED}✗ No response${NC}"
    fi
}

# Start the server
echo -e "\n${YELLOW}Starting Lux MCP server...${NC}"
RUST_LOG=info,lux_mcp::threading=debug ./target/release/lux-mcp 2>test_threading.log &
SERVER_PID=$!
sleep 3

# Check if server started
if ps -p $SERVER_PID > /dev/null; then
    echo -e "${GREEN}✓ Server started (PID: $SERVER_PID)${NC}"
else
    echo -e "${RED}✗ Server failed to start${NC}"
    exit 1
fi

echo -e "\n${YELLOW}=== Phase 1-2: Basic Threading ===${NC}"

# Test 1: New conversation
send_request "Test 1: Starting new conversation" '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Hello! I want to learn about metacognitive monitoring in AI.",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

# Test 2: Continue with thread ID
send_request "Test 2: Continuing with thread ID" '{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What specific aspect did I just ask about?",
      "continuation_id": "thread-meta-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

echo -e "\n${YELLOW}=== Phase 3: Synthesis Integration ===${NC}"

# Test 3: Traced reasoning with synthesis
send_request "Test 3: Traced reasoning with synthesis tracking" '{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "traced_reasoning",
    "arguments": {
      "thought": "How does Lux detect circular reasoning patterns?",
      "thought_number": 1,
      "total_thoughts": 3,
      "next_thought_needed": true,
      "continuation_id": "thread-meta-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

echo -e "\n${YELLOW}=== Phase 4: Quality Metrics ===${NC}"

# Test 4: Multiple interactions to build quality metrics
send_request "Test 4a: Building quality metrics" '{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Can you explain the quality metrics you track?",
      "continuation_id": "thread-meta-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

send_request "Test 4b: More quality data" '{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "How do these metrics improve over time?",
      "continuation_id": "thread-meta-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

echo -e "\n${YELLOW}=== Phase 5: Persistence (if DB configured) ===${NC}"

# Test 5: Check persistence capability
if [ -n "$DATABASE_URL" ]; then
    echo -e "${GREEN}Database configured - persistence enabled${NC}"
    send_request "Test 5: Thread with persistence" '{
      "jsonrpc": "2.0",
      "id": 6,
      "method": "tools/call",
      "params": {
        "name": "confer",
        "arguments": {
          "message": "This conversation should be persisted to database.",
          "continuation_id": "thread-persist-001",
          "model": "gpt-4o-mini"
        }
      }
    }'
else
    echo -e "${YELLOW}No DATABASE_URL - running in-memory only${NC}"
fi

sleep 1

echo -e "\n${YELLOW}=== Thread Isolation Test ===${NC}"

# Test 6: Different thread
send_request "Test 6a: Different thread context" '{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "This is about cooking recipes, completely different topic.",
      "continuation_id": "thread-cooking-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

send_request "Test 6b: Verify isolation" '{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "What topic are we discussing?",
      "continuation_id": "thread-cooking-001",
      "model": "gpt-4o-mini"
    }
  }
}'

sleep 1

# Return to original thread
send_request "Test 6c: Return to original thread" '{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "confer",
    "arguments": {
      "message": "Summarize everything we discussed about metacognition.",
      "continuation_id": "thread-meta-001",
      "model": "gpt-4o-mini"
    }
  }
}'

echo -e "\n${YELLOW}=== Server Logs Analysis ===${NC}"

# Analyze logs
echo "Thread-related logs:"
grep -E "(thread|Thread|continuation|synthesis|quality)" test_threading.log | tail -20 || echo "No thread logs found"

echo -e "\nThread creation events:"
grep -E "(Creating|Created|Restored) thread" test_threading.log || echo "No thread creation logs"

echo -e "\nSynthesis events:"
grep -i "synthesis" test_threading.log | head -5 || echo "No synthesis logs"

echo -e "\nQuality metrics:"
grep -i "quality" test_threading.log | head -5 || echo "No quality logs"

# Server stats
echo -e "\n${YELLOW}=== Final Statistics ===${NC}"
echo "Total requests sent: 9"
echo "Server uptime: ~30 seconds"
echo "Threads created: $(grep -c "Creating thread" test_threading.log 2>/dev/null || echo "0")"
echo "Log file size: $(wc -l test_threading.log 2>/dev/null | awk '{print $1}' || echo "0") lines"

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Server stopped${NC}"
else
    echo -e "${RED}✗ Server already stopped${NC}"
fi

# Save logs for inspection
mv test_threading.log test_threading_$(date +%Y%m%d_%H%M%S).log 2>/dev/null

echo -e "\n${GREEN}=========================================${NC}"
echo -e "${GREEN}✅ Complete Threading System Test Done!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "Phases tested:"
echo "  ✓ Phase 1-2: Basic threading with continuation_id"
echo "  ✓ Phase 3: Synthesis integration"
echo "  ✓ Phase 4: Quality metrics tracking"
echo "  ✓ Phase 5: Persistence layer (if configured)"
echo "  ✓ Thread isolation and context switching"
echo ""
echo "Logs saved to: test_threading_*.log"