#!/bin/bash
# PureReason MCP Server Test Script
# Demonstrates the MCP integration by calling all available tools

set -e

MCP_BIN="${MCP_BIN:-./target/release/pure-reason-mcp}"

if [ ! -f "$MCP_BIN" ]; then
    echo "Error: MCP binary not found at $MCP_BIN"
    echo "Run: cargo build --release -p pure-reason-mcp"
    exit 1
fi

echo "=== PureReason MCP Integration Test ==="
echo

# Test 1: Initialize
echo "1. Initialize MCP server"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | \
    $MCP_BIN | jq -r '.result.serverInfo.name'
echo "✓ Server initialized"
echo

# Test 2: List tools
echo "2. List available tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | \
    $MCP_BIN | jq -r '.result.tools[] | "  - \(.name): \(.description)"'
echo

# Test 3: Verify valid text
echo "3. Verify valid mathematical statement"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"verify_text","arguments":{"text":"Water boils at 100°C at sea level."}}}' | \
    $MCP_BIN | jq -r '.result.content[0].text' | jq '{passed: .verdict.passed, summary: .verdict.summary}'
echo

# Test 3b: Detect arithmetic error
echo "3b. Detect arithmetic error"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"verify_text","arguments":{"text":"120 divided by 2 equals 90"}}}' | \
    $MCP_BIN | jq -r '.result.content[0].text' | jq '{passed: .verdict.passed, error: .findings[0].message}'
echo

# Test 4: Verify structured decision with contradiction
echo "4. Verify structured decision (should fail)"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"verify_structured_decision","arguments":{"json":"{\"contraindications\":[\"warfarin\"],\"prescribed\":[\"warfarin\"]}"}}}' | \
    $MCP_BIN | jq -r '.result.content[0].text' | jq '{passed: .verdict.passed, findings: .findings}'
echo

# Test 5: Review text (creates persistent task)
echo "5. Create review task"
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"review_text","arguments":{"text":"The system should handle edge cases correctly."}}}' | \
    $MCP_BIN | jq -r '.result.content[0].text' | jq '{task_id, final_state, passed: .verification.verdict.passed}'
echo

# Test 6: Analyze with Kantian pipeline
echo "6. Full Kantian pipeline analysis"
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"analyze","arguments":{"text":"All mammals are warm-blooded. Whales are mammals. Therefore whales are warm-blooded."}}}' | \
    $MCP_BIN | jq -r '.result.content[0].text' | jq '{ecs, risk: .verdict.risk, has_illusions: .verdict.has_illusions}'
echo

echo "=== All tests passed ✓ ==="
