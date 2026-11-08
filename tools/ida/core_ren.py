# usage: python tools/ida/core_ren.py ADDR=Name [ADDR=Name ...]   (client side, uses the rename tool)
import sys, os, json, subprocess
ops = [{"addr": "0x" + a.split("=", 1)[0], "name": a.split("=", 1)[1]} for a in sys.argv[1:]]
root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
r = subprocess.run([os.path.join(root, ".venv-ida", "Scripts", "python"), os.path.join(root, "tools", "mcp.py"),
                    "rename", json.dumps({"batch": {"func": ops}})], capture_output=True, text=True)
out = r.stdout
try:
    j = json.loads(out)
    bad = [x for x in json.dumps(j).split("},") if "error" in x.lower() and "null" not in x.lower()]
    print("ok" if not bad else "\n".join(bad[:20]))
except Exception:
    print(out[:3000], r.stderr[:2000])
