import sys, argparse
from typing import Dict

from fastapi import FastAPI
import uvicorn

app = FastAPI(title="Tauri Python Sidecar")

@app.post("/api/greet")
async def greet(name: Dict[str, str]) -> Dict[str, str]:
    """
    Implements the same greeting functionality as the Tauri Rust backend.
    Expects a JSON object with a "name" field.
    """
    return {"response": f"Hello, {name['name']}! You've been greeted from Python!"}

@app.get("/health")
async def health() -> Dict[str, str]:
    """Health check endpoint."""
    return {"status": "ok"}

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Tauri Python Sidecar")
    parser.add_argument("--port", type=int, default=8000, help="Port to listen on")
    args = parser.parse_args(sys.argv[1:])
    uvicorn.run(app, host="127.0.0.1", port=args.port)
