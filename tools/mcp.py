"""minimal client for the headless idalib-mcp server (streamable http).

usage: python tools/mcp.py list
       python tools/mcp.py <tool> '<json args>'
       python tools/mcp.py py <file.py>      # run an idapython script via execute/py tool
"""
import json
import sys
import urllib.request

URL = "http://127.0.0.1:8745/mcp"
_sid = None


def _post(payload):
    global _sid
    req = urllib.request.Request(URL, data=json.dumps(payload).encode(), method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Accept", "application/json, text/event-stream")
    if _sid:
        req.add_header("Mcp-Session-Id", _sid)
    with urllib.request.urlopen(req, timeout=3600) as r:
        _sid = r.headers.get("Mcp-Session-Id") or _sid
        body = r.read().decode("utf-8", "replace")
    if not body.strip():
        return None
    if body.lstrip().startswith("{"):
        return json.loads(body)
    # sse: take the last data line
    data = [l[5:].strip() for l in body.splitlines() if l.startswith("data:")]
    return json.loads(data[-1]) if data else None


def init():
    _post({"jsonrpc": "2.0", "id": 0, "method": "initialize",
           "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                      "clientInfo": {"name": "cli", "version": "1"}}})
    _post({"jsonrpc": "2.0", "method": "notifications/initialized"})


def call(tool, args=None):
    args = dict(args or {})
    if tool not in ("idb_list", "idb_open") and "database" not in args:
        args["database"] = _db()
    r = _post({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
               "params": {"name": tool, "arguments": args}})
    if "error" in r:
        raise RuntimeError(r["error"])
    res = r["result"]
    if "structuredContent" in res and res["structuredContent"] is not None:
        return res["structuredContent"]
    return "\n".join(c.get("text", "") for c in res.get("content", []))


_dbid = None


def _db():
    global _dbid
    if _dbid is None:
        _dbid = call("idb_list")["sessions"][0]["session_id"]
    return _dbid


def main():
    init()
    if sys.argv[1] == "list":
        r = _post({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        for t in r["result"]["tools"]:
            print(t["name"], "-", (t.get("description") or "").split("\n")[0][:110])
        return
    if sys.argv[1] == "schema":
        r = _post({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        for t in r["result"]["tools"]:
            if t["name"] == sys.argv[2]:
                print(json.dumps(t, indent=1))
        return
    if sys.argv[1] == "py":
        import os
        out = call("py_exec_file", {"file_path": os.path.abspath(sys.argv[2])})
        if isinstance(out, dict):
            for k in ("stdout", "result", "stderr"):
                if out.get(k):
                    print(out[k])
            return
    else:
        out = call(sys.argv[1], json.loads(sys.argv[2]) if len(sys.argv) > 2 else {})
    print(out if isinstance(out, str) else json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
