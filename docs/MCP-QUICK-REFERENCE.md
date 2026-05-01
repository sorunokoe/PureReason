# PureReason MCP Quick Reference

## Installation

```bash
cargo build --release -p pure-reason-mcp
# Binary: ./target/release/pure-reason-mcp
```

## Claude Desktop Configuration

**File**: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)

```json
{
  "mcpServers": {
    "pure-reason": {
      "command": "/path/to/PureReason/target/release/pure-reason-mcp"
    }
  }
}
```

## Available Tools

| Tool | Purpose | Input | Best For |
|------|---------|-------|----------|
| `verify_text` | Quick verification | `{text: string}` | Fast checks, low overhead |
| `verify_structured_decision` | JSON validation | `{json: string}` | Medical, financial, critical decisions |
| `review_text` | Persistent review | `{text: string}` | Auditable workflows, human escalation |
| `review_structured_decision` | Persistent JSON review | `{json: string}` | High-stakes structured decisions |
| `analyze` | Full pipeline | `{text: string}` | Complete Kantian analysis |
| `certify` | Generate certificate | `{text: string}` | Content-addressed validation |
| `regulate` | Rewrite overconfident text | `{text: string}` | Fix epistemic overreach |
| `validate` | Quick dialectical check | `{text: string}` | Fast validation path |

## Key Differences

**verify_* vs review_***:
- `verify_*`: Stateless, fast, no persistence
- `review_*`: Creates persistent task/trace/evidence records in `~/.pure-reason/agent-state/`

**verify_text vs verify_structured_decision**:
- `verify_text`: Plain text, general content
- `verify_structured_decision`: JSON with structured fields (contraindications, constraints)

## Common Patterns

### Pre-Action Verification
```
Agent plans action → call verify_text → if passed, execute → if failed, revise
```

### Structured Decision Review
```
Generate decision JSON → call verify_structured_decision → check findings → iterate until passed
```

### Human-in-the-Loop Escalation
```
High-risk action → call review_text → creates persistent task → human reviews → agent proceeds
```

## Risk Levels

- **Safe** / **passed: true**: Proceed with confidence
- **Low**: Minor issues, likely okay
- **Medium**: Review recommended
- **High**: Don't proceed without human review
- **Critical**: Block action, escalate immediately

## Persistent State

**Location**: `~/.pure-reason/agent-state/`

**Files**:
- `tasks.sqlite3` — Review tasks
- `traces.sqlite3` — Execution traces
- `evidence.db` — Verification evidence

**Override**: Set `PURE_REASON_STATE_DIR` environment variable

## Performance

- Cold start: ~50-100ms
- Warm verification: ~10-30ms
- Memory: ~50MB baseline

## Testing

```bash
# Quick test
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/pure-reason-mcp

# Full test suite
./examples/mcp-test.sh
```

## Troubleshooting

**Server not appearing**: Check binary path, restart agent, check logs

**Database locked**: Stop other instances, set unique `PURE_REASON_STATE_DIR`

**Too strict/lenient**: Use structured decisions for high-stakes, review_* for persistence

## Security

✓ 100% local — no network calls  
✓ No API keys required  
✓ Works offline  
✓ No telemetry  
✓ Full audit trail in sqlite

## Learn More

- Integration guide: [`docs/MCP-INTEGRATION.md`](./MCP-INTEGRATION.md)
- Capabilities: [`docs/CAPABILITIES.md`](./CAPABILITIES.md)
- Roadmap: [`docs/ADR-002.md`](./ADR-002.md)
