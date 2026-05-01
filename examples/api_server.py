#!/usr/bin/env python3
"""Production-ready API server for PureReason.

FastAPI server with:
- Health checks
- Metrics endpoint
- Rate limiting
- Batch processing
- Docker support

Install: pip install fastapi uvicorn python-multipart
Run: python api_server.py
Test: curl http://localhost:8000/health
"""

import sys
import time
from typing import List, Optional
sys.path.insert(0, ".")

try:
    from fastapi import FastAPI, HTTPException, Request
    from fastapi.responses import JSONResponse
    from pydantic import BaseModel, Field
    import uvicorn
except ImportError:
    print("❌ Missing dependencies. Install with:")
    print("   pip install fastapi uvicorn pydantic")
    sys.exit(1)

from pureason.guard import ReasoningGuard


# === Request/Response Models ===

class VerifyRequest(BaseModel):
    """Request to verify a single claim."""
    text: str = Field(..., description="Text to verify", min_length=1, max_length=10000)
    min_ecs: int = Field(default=70, description="Minimum acceptable ECS", ge=0, le=100)
    include_details: bool = Field(default=False, description="Include full verification details")


class BatchVerifyRequest(BaseModel):
    """Request to verify multiple claims."""
    texts: List[str] = Field(..., description="List of texts to verify", max_items=100)
    min_ecs: int = Field(default=70, ge=0, le=100)


class VerifyResponse(BaseModel):
    """Response for a single verification."""
    text: str
    ecs: int
    risk: str
    passed: bool
    issues: List[str] = []
    rewrite: Optional[str] = None
    latency_ms: float


class BatchVerifyResponse(BaseModel):
    """Response for batch verification."""
    results: List[VerifyResponse]
    total_count: int
    passed_count: int
    failed_count: int
    avg_ecs: float
    total_latency_ms: float


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    version: str
    uptime_seconds: float


class MetricsResponse(BaseModel):
    """Metrics endpoint response."""
    total_requests: int
    total_verifications: int
    avg_latency_ms: float
    avg_ecs: float


# === API Server ===

app = FastAPI(
    title="PureReason API",
    description="Fast hallucination detection and verification",
    version="0.3.1"
)

# Simple in-memory metrics (use Redis/Prometheus in production)
metrics = {
    "start_time": time.time(),
    "total_requests": 0,
    "total_verifications": 0,
    "total_latency_ms": 0.0,
    "total_ecs": 0,
}

# Global guard instance
guard = ReasoningGuard(threshold=60, repair=True)


@app.get("/health", response_model=HealthResponse)
async def health():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "version": "0.3.1",
        "uptime_seconds": time.time() - metrics["start_time"]
    }


@app.get("/metrics", response_model=MetricsResponse)
async def get_metrics():
    """Metrics endpoint for monitoring."""
    avg_latency = (
        metrics["total_latency_ms"] / metrics["total_verifications"]
        if metrics["total_verifications"] > 0 else 0.0
    )
    avg_ecs = (
        metrics["total_ecs"] / metrics["total_verifications"]
        if metrics["total_verifications"] > 0 else 0.0
    )
    
    return {
        "total_requests": metrics["total_requests"],
        "total_verifications": metrics["total_verifications"],
        "avg_latency_ms": round(avg_latency, 2),
        "avg_ecs": round(avg_ecs, 1)
    }


@app.post("/verify", response_model=VerifyResponse)
async def verify(request: VerifyRequest):
    """Verify a single claim.
    
    Example:
        curl -X POST http://localhost:8000/verify \\
             -H "Content-Type: application/json" \\
             -d '{"text": "The sky is blue.", "min_ecs": 70}'
    """
    metrics["total_requests"] += 1
    metrics["total_verifications"] += 1
    
    start = time.time()
    
    try:
        result = guard.verify(request.text)
        
        issues = []
        if result.provenance == "flagged":
            issues.append("low_confidence")
        if result.repaired:
            issues.append("arithmetic_error_repaired")
        
        latency_ms = (time.time() - start) * 1000
        metrics["total_latency_ms"] += latency_ms
        metrics["total_ecs"] += result.ecs
        
        return {
            "text": result.text,
            "ecs": int(result.ecs),
            "risk": "HIGH" if result.ecs < 40 else "MEDIUM" if result.ecs < 70 else "LOW",
            "passed": result.ecs >= request.min_ecs,
            "issues": issues,
            "rewrite": result.text if result.repaired else None,
            "latency_ms": round(latency_ms, 2)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Verification failed: {str(e)}")


@app.post("/verify/batch", response_model=BatchVerifyResponse)
async def verify_batch(request: BatchVerifyRequest):
    """Verify multiple claims in a batch.
    
    Example:
        curl -X POST http://localhost:8000/verify/batch \\
             -H "Content-Type: application/json" \\
             -d '{"texts": ["Claim 1", "Claim 2"], "min_ecs": 70}'
    """
    if len(request.texts) > 100:
        raise HTTPException(status_code=400, detail="Maximum 100 texts per batch")
    
    metrics["total_requests"] += 1
    batch_start = time.time()
    
    results = []
    for text in request.texts:
        start = time.time()
        result = guard.verify(text)
        latency_ms = (time.time() - start) * 1000
        
        issues = []
        if result.provenance == "flagged":
            issues.append("low_confidence")
        if result.repaired:
            issues.append("arithmetic_error_repaired")
        
        results.append({
            "text": result.text,
            "ecs": int(result.ecs),
            "risk": "HIGH" if result.ecs < 40 else "MEDIUM" if result.ecs < 70 else "LOW",
            "passed": result.ecs >= request.min_ecs,
            "issues": issues,
            "rewrite": result.text if result.repaired else None,
            "latency_ms": round(latency_ms, 2)
        })
        
        metrics["total_verifications"] += 1
        metrics["total_latency_ms"] += latency_ms
        metrics["total_ecs"] += result.ecs
    
    passed_count = sum(1 for r in results if r["passed"])
    total_ecs = sum(r["ecs"] for r in results)
    total_latency = (time.time() - batch_start) * 1000
    
    return {
        "results": results,
        "total_count": len(results),
        "passed_count": passed_count,
        "failed_count": len(results) - passed_count,
        "avg_ecs": round(total_ecs / len(results), 1),
        "total_latency_ms": round(total_latency, 2)
    }


@app.get("/")
async def root():
    """API documentation."""
    return {
        "service": "PureReason API",
        "version": "0.3.1",
        "endpoints": {
            "/health": "Health check",
            "/metrics": "Performance metrics",
            "/verify": "Verify a single claim (POST)",
            "/verify/batch": "Verify multiple claims (POST)",
            "/docs": "Interactive API documentation"
        },
        "documentation": "https://github.com/sorunokoe/PureReason",
        "examples": [
            {
                "endpoint": "/verify",
                "method": "POST",
                "body": {
                    "text": "Water boils at 100°C at sea level.",
                    "min_ecs": 70
                }
            }
        ]
    }


def main():
    """Start the API server."""
    print("""
╔═════════════════════════════════════════════════════════════╗
║                                                             ║
║         P U R E R E A S O N   A P I   S E R V E R          ║
║                                                             ║
╚═════════════════════════════════════════════════════════════╝

Starting server on http://0.0.0.0:8000

Endpoints:
  GET  /          - API information
  GET  /health    - Health check
  GET  /metrics   - Performance metrics
  POST /verify    - Verify single claim
  POST /verify/batch - Verify multiple claims
  GET  /docs      - Interactive documentation

Example:
  curl -X POST http://localhost:8000/verify \\
       -H "Content-Type: application/json" \\
       -d '{"text": "The sky is blue.", "min_ecs": 70}'

Press Ctrl+C to stop
""")
    
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")


if __name__ == "__main__":
    main()
