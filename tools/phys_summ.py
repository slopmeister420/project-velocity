# physics agent: one line per function of a phys_dump file: addr, lines, signature, strings used
import re, sys
src = open(sys.argv[1], encoding="utf-8").read()
parts = re.split(r"\n// ==== ([0-9A-F]{8}) (\S+)\n", src)
for i in range(1, len(parts), 3):
    addr, name, body = parts[i], parts[i + 1], parts[i + 2]
    strs = re.findall(r'"((?:[^"\\]|\\.)*)"', body)
    strs = [s for s in strs if ".cpp" not in s and ".h" not in s[-3:]]
    sig = body.split("\n", 1)[0][:100]
    print(addr, len(body.splitlines()), sig, "|", " ; ".join(dict.fromkeys(strs))[:400])
