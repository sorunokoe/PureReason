# PureReason MCP Integration Guide

## Overview

PureReason provides an **MCP (Model Context Protocol) server** that allows frontier AI agents (Claude Code, GitHub Copilot, Cursor, etc.) to call PureReason as a local verification and assurance layer.

The MCP server exposes PureReason's core capabilities through JSON-RPC 2.0 over stdio, enabling agents to:

- Verify reasoning chains and check for logical errors
- Detect contradictions, overconfidence, and hallucination risks
- Review structured decisions (e.g., medical prescriptions, financial recommendations)
- Generate content-addressed validation certificates
- Regulate overconfident language into more defensible forms
- Track verification tasks with persistent local state

## Architecture

```
┌─────────────────────┐
│  Frontier Agent     │
│  (Claude Code)      │
└──────────┬──────────┘
           │ MCP (stdio)
           ▼
┌─────────────────────┐
│  PureReason MCP     │
│  pure-reason-mcp    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐       ┌─────────────────────┐
│  Verification       │◄──────┤  Local State        │
│  • Verifier Service │       │  ~/.pure-reason/    │
│  • Kantian Pipeline │       │  ├── tasks.sqlite3  │
│  • Runtime Executor │       │  ├── traces.sqlite3 │
│  • Evidence Store   │       │  └── evidence.db    │
└─────────────────────┘       └─────────────────────┘
```

## Installation

### 1. Build the MCP server

```bash
cd /path/to/PureReason
cargo build --release -p pure-reason-mcp
```

The binary will be at: `target/release/pure-reason-mcp`

### 2. Install globally (optional)

```bash
cargo install --path crates/pure-reason-mcp --locked
```

This makes `pure-reason-mcp` available globally in your PATH.

## Configuration

### Claude Desktop

Add to your Claude Desktop MCP configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`  
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "pure-reason": {
      "command": "/path/to/PureReason/target/release/pure-reason-mcp"
    }
  }
}
```

Or if installed globally:

```json
{
  "mcpServers": {
    "pure-reason": {
      "command": "pure-reason-mcp"
    }
  }
}
```

### GitHub Copilot CLI

Add to `~/.copilot/config.yml`:

```yaml
mcp_servers:
  pure-reason:
    command: /path/to/PureReason/target/release/pure-reason-mcp
```

### Cursor / Other MCP-compatible agents

Refer to your agent's MCP configuration documentation. The command is always:

```bash
pure-reason-mcp
```

## Available Tools

The MCP server exposes 8 tools:

### Core Verification Tools

#### `verify_text`
Quick verification of plain text content.

**Input**:
```json
{
  "text": "The patient must have cancer."
}
```

**Output**:
```json
{
  "verdict": {
    "passed": false,
    "risk_level": "high",
    "confidence": 34
  },
  "findings": [
    {
      "kind": "overconfidence",
      "severity": "high",
      "location": "must have cancer",
      "explanation": "Certainty overreach without supporting evidence"
    }
  ],
  "regulated_text": "The patient has findings consistent with possible malignancy."
}
```

#### `verify_structured_decision`
Verify structured JSON decisions (e.g., medical prescriptions, financial recommendations).

**Input**:
```json
{
  "json": "{\"contraindications\":[\"warfarin\"],\"prescribed\":[\"warfarin\"]}"
}
```

**Output**:
```json
{
  "verdict": {
    "passed": false,
    "risk_level": "critical"
  },
  "findings": [
    {
      "kind": "contradiction",
      "severity": "critical",
      "explanation": "Prescribed medication is contraindicated"
    }
  ]
}
```

### Review Workflow Tools

#### `review_text`
Create a full review task with persistent state tracking.

**Input**:
```json
{
  "text": "The sky is blue."
}
```

**Output**:
```json
{
  "task_id": "task_01J9...",
  "trace_id": "trace_01J9...",
  "final_state": "completed",
  "verification": {
    "verdict": {
      "passed": true,
      "confidence": 92
    }
  }
}
```

#### `review_structured_decision`
Same as `review_text` but for structured JSON decisions.

### Legacy Kantian Pipeline Tools

#### `analyze`
Full Kantian pipeline analysis (retained for compatibility).

**Input**:
```json
{
  "text": "Water boils at 100°C at sea level."
}
```

**Output**:
```json
{
  "ecs": 88,
  "verdict": {
    "risk": "low",
    "has_illusions": false,
    "has_contradictions": false
  },
  "regulated_text": "Water boils at 100°C at sea level.",
  "summary": "Factual claim with domain-calibrated confidence"
}
```

#### `certify`
Generate a content-addressed validation certificate.

#### `regulate`
Convert overconfident language to regulative form.

#### `validate`
Quick dialectical validation (fast path).

## Persistent State

PureReason maintains persistent local state for review workflows:

- **Location**: `~/.pure-reason/agent-state/` (default)
- **Override**: Set `PURE_REASON_STATE_DIR` environment variable
- **Contents**:
  - `tasks.sqlite3` — Review task records
  - `traces.sqlite3` — Execution trace events
  - `evidence.db` — Evidence records with verification results

This enables:
- Auditable review history
- Multi-turn review workflows
- Human approval escalation paths
- Post-hoc analysis and debugging

## Usage Examples

### Example 1: Verify a code change plan

```
Agent: I plan to refactor the authentication module by removing the deprecated OAuth1 handler and migrating all users to OAuth2.

[Agent calls PureReason verify_text tool]

PureReason Response:
{
  "verdict": { "passed": false, "risk_level": "medium" },
  "findings": [
    {
      "kind": "unstated_assumption",
      "severity": "medium",
      "explanation": "Plan assumes all users can migrate to OAuth2, but doesn't mention backward compatibility or migration path for existing OAuth1 tokens"
    }
  ]
}

Agent: Based on PureReason's finding, I should add a migration plan and token transition strategy before proceeding.
```

### Example 2: Review a medical decision

```
Agent: Based on the patient's symptoms, I recommend prescribing aspirin.

[Agent calls review_structured_decision with full context]

PureReason Response:
{
  "task_id": "task_01J9XYZ",
  "final_state": "awaiting_review",
  "verification": {
    "verdict": { "passed": false, "risk_level": "high" },
    "findings": [
      {
        "kind": "missing_critical_check",
        "severity": "high",
        "explanation": "No contraindication check for bleeding disorders or anticoagulant interactions"
      }
    ]
  }
}

Agent: PureReason flagged a critical safety check. I should verify contraindications before recommending aspirin.
```

### Example 3: Arithmetic reasoning verification

```
Agent: A train travels 120 miles in 2 hours, so its speed is 90 mph.

[Agent calls verify_text tool]

PureReason Response:
{
  "verdict": { "passed": false },
  "findings": [
    {
      "kind": "arithmetic_error",
      "severity": "high",
      "location": "speed is 90 mph",
      "explanation": "120 miles / 2 hours = 60 mph, not 90 mph"
    }
  ]
}

Agent: Correcting my calculation: the speed is 60 mph.
```

## Integration Patterns

### Pattern 1: Pre-action verification

```
1. Agent plans an action (code change, recommendation, command)
2. Agent calls PureReason to verify the plan
3. If verification passes → proceed with action
4. If verification fails → revise plan or escalate to human
```

### Pattern 2: Post-generation review

```
1. Agent generates output (spec, document, analysis)
2. Agent calls PureReason to review the output
3. If review passes → return to user
4. If review fails → iterate with PureReason's findings
```

### Pattern 3: Iterative refinement

```
1. Agent generates initial solution
2. Agent reviews with PureReason
3. Agent refines based on findings
4. Repeat until verification passes
5. Return final verified solution
```

### Pattern 4: Human-in-the-loop escalation

```
1. Agent plans high-risk action (delete data, financial transaction)
2. Agent calls review_text to create persistent review task
3. If risk_level > threshold → task state = "awaiting_review"
4. Human reviews task via dashboard or CLI
5. Human approves/rejects
6. Agent proceeds based on human decision
```

## Testing the Integration

### 1. Test MCP server directly

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/pure-reason-mcp
```

Expected output:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {"name": "analyze", "description": "..."},
      {"name": "verify_text", "description": "..."},
      ...
    ]
  }
}
```

### 2. Test verification tool

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"verify_text","arguments":{"text":"The patient must have cancer."}}}' | ./target/release/pure-reason-mcp
```

### 3. Test from Claude Desktop

After configuring MCP:

1. Restart Claude Desktop
2. Open a new conversation
3. Ask: "Can you verify this reasoning: 120 divided by 2 equals 90"
4. Claude should use the PureReason MCP tools
5. Check Claude's response includes PureReason's verification findings

## Troubleshooting

### MCP server not appearing in agent

1. Check configuration file path is correct
2. Verify binary path is absolute or in PATH
3. Restart agent application
4. Check agent logs for MCP initialization errors

### State directory errors

If you see `PURE_REASON_STATE_DIR` errors:

```bash
export PURE_REASON_STATE_DIR=~/.pure-reason/agent-state
mkdir -p $PURE_REASON_STATE_DIR
```

Or set in MCP config:

```json
{
  "mcpServers": {
    "pure-reason": {
      "command": "pure-reason-mcp",
      "env": {
        "PURE_REASON_STATE_DIR": "/Users/your-user/.pure-reason/agent-state"
      }
    }
  }
}
```

### Verification is too strict/lenient

PureReason uses domain-aware calibration. If results don't match expectations:

1. Check the domain context in your input
2. Provide structured decisions for high-stakes scenarios
3. Use `review_*` tools for persistent task tracking
4. Adjust thresholds in your agent's integration logic

## Performance

- **Cold start**: ~50-100ms (first verification after MCP server starts)
- **Warm verification**: ~10-30ms (subsequent verifications)
- **Structured decision review**: ~20-50ms
- **Memory footprint**: ~50MB baseline + ~5MB per active task

All operations are **deterministic** and **local** — no network calls, no API keys required.

## Security & Privacy

- **100% local** — no data leaves your machine
- **No telemetry** — no phone-home, no analytics
- **No credentials required** — works offline
- **Audit trail** — all verifications are logged to local sqlite databases

## Limitations

- Optimized for **claim-level reasoning** (<10K tokens), not long-context analysis
- **Not a problem solver** — verifies existing solutions, doesn't generate new ones
- **Not a frontier model** — complements LLMs, doesn't replace them
- **No general knowledge** — focuses on logical coherence and risk detection

## Next Steps

- **ADR-002**: Read the [Agentic Reasoning Assurance Roadmap](./ADR-002.md)
- **Capabilities**: See [CAPABILITIES.md](./CAPABILITIES.md) for measured capabilities
- **Benchmarks**: Check [BENCHMARK.md](./BENCHMARK.md) for current performance data
- **CLI**: Try the CLI: `pure-reason review --help`

## Support

- **Issues**: https://github.com/sorunokoe/PureReason/issues
- **Discussions**: https://github.com/sorunokoe/PureReason/discussions
- **Contributing**: See [CONTRIBUTING.md](../.github/CONTRIBUTING.md)
