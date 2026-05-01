# Quick Start Examples

This directory contains practical examples for using PureReason in real-world applications.

## 🚀 Quick Start

### 1. Simple Verification (5 minutes)

Verify claims with PureReason:

```bash
python examples/simple_verification.py
```

**What you'll see:**
- ✅ Factual claims get high ECS scores (80-90)
- ⚠️  Overconfident claims get flagged (30-60)
- ❌ Contradictions get rejected (<30)

### 2. LangChain Integration (10 minutes)

Use PureReason as a verification layer in LangChain:

```bash
pip install langchain langchain-openai
python examples/langchain_integration.py
```

**Key pattern:**
```python
from pureason.reasoning.chain import verify_chain

# Verify LLM output
result = verify_chain(llm_output)
if result.ecs >= 70:
    # Use the output
else:
    # Reject or retry
```

### 3. Production API Server (15 minutes)

Deploy PureReason as a microservice:

```bash
# Install dependencies
pip install fastapi uvicorn

# Start server
python examples/api_server.py

# Test it
curl -X POST http://localhost:8000/verify \
     -H "Content-Type: application/json" \
     -d '{"text": "The sky is blue.", "min_ecs": 70}'
```

**Response:**
```json
{
  "text": "The sky is blue.",
  "ecs": 85,
  "risk": "LOW",
  "passed": true,
  "issues": [],
  "latency_ms": 4.2
}
```

## 🐳 Docker Deployment

Build and run with Docker:

```bash
# Build image
docker build -f examples/Dockerfile.api -t pureason-api .

# Run container
docker run -p 8000:8000 pureason-api

# Test health
curl http://localhost:8000/health
```

## 📊 API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/metrics` | GET | Performance metrics |
| `/verify` | POST | Verify single claim |
| `/verify/batch` | POST | Verify up to 100 claims |
| `/docs` | GET | Interactive API docs |

## 🎯 Integration Patterns

### Pattern 1: Guard Rails
```python
def safe_llm_call(prompt):
    output = llm.generate(prompt)
    verification = verify_chain(output)
    
    if verification.ecs < 70:
        # Reject low-confidence output
        raise ValueError("Output failed verification")
    
    return output
```

### Pattern 2: Confidence Scoring
```python
def scored_generation(prompt):
    output = llm.generate(prompt)
    verification = verify_chain(output)
    
    return {
        "text": output,
        "confidence": verification.ecs / 100,
        "safe_to_use": verification.ecs >= 70
    }
```

### Pattern 3: Auto-Correction
```python
def self_correcting_llm(prompt):
    output = llm.generate(prompt)
    verification = verify_chain(output)
    
    if verification.ecs < 70 and verification.rewrites:
        # Use PureReason's rewrite
        return verification.rewrites[0]
    
    return output
```

## 🔧 Configuration

### ECS Thresholds

Choose based on your risk tolerance:

| Risk Level | Domain | Min ECS |
|------------|--------|---------|
| **Critical** | Medical, Legal, Financial | 85+ |
| **High** | Business, Education | 75+ |
| **Medium** | General knowledge | 65+ |
| **Low** | Creative, Opinion | 50+ |

### Performance Tuning

- **Latency**: <5ms per verification (typical)
- **Throughput**: 200+ verifications/second
- **Batch size**: Up to 100 claims per request

## 📚 More Examples

Coming soon:
- Jupyter notebook tutorials
- TypeScript/JavaScript integration
- Streaming verification
- Custom domain calibration

## 💡 Tips

1. **Always verify** LLM outputs for critical applications
2. **Monitor ECS scores** over time to track model drift
3. **Use batch endpoints** for high throughput
4. **Set appropriate thresholds** based on domain risk
5. **Enable rewrites** for automatic correction

## 🆘 Troubleshooting

**"ModuleNotFoundError: No module named 'pureason'"**
```bash
pip install -e .
```

**"API server won't start"**
```bash
pip install fastapi uvicorn pydantic
```

**"Verification takes too long"**
- Use batch endpoint for multiple claims
- Consider caching for repeated verification
- Check system resources (CPU, memory)

## 📖 Documentation

- [Full Documentation](../docs/README.md)
- [Benchmarks](../docs/BENCHMARK.md)
- [Architecture](../docs/CAPABILITIES.md)
- [MCP Integration](../docs/MCP-QUICK-REFERENCE.md)

## 🤝 Contributing

See [CONTRIBUTING.md](../docs/CONTRIBUTING.md) for guidelines.

## 📄 License

Apache 2.0 - See [LICENSE](../LICENSE) for details.
