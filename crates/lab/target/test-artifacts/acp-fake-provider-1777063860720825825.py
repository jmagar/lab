import json
import os
import sys

session_id = f"stub-session-{os.getpid()}"

for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue

    msg = json.loads(raw)
    req_id = msg.get("id")
    method = msg.get("method")
    params = msg.get("params") or {}

    if method == "initialize":
        result = {
            "protocolVersion": params.get("protocolVersion", "v1"),
            "agentCapabilities": {},
            "authMethods": [],
            "agentInfo": {
                "name": "stub-acp",
                "version": "0.0.0",
                "title": "Stub ACP",
            },
        }
        print(json.dumps({"jsonrpc": "2.0", "id": req_id, "result": result}), flush=True)
    elif method == "session/new":
        result = {"sessionId": session_id, "configOptions": []}
        print(json.dumps({"jsonrpc": "2.0", "id": req_id, "result": result}), flush=True)
    elif method == "session/prompt":
        print(json.dumps({"jsonrpc": "2.0", "id": req_id, "result": {}}), flush=True)
    elif req_id is not None:
        error = {"code": -32601, "message": "method not found"}
        print(json.dumps({"jsonrpc": "2.0", "id": req_id, "error": error}), flush=True)
