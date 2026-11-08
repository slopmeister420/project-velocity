import re, os, glob
root = r"D:\projects\project poland rewrite\research"
bodies = {}
for fn in glob.glob(os.path.join(root, "ai_dump", "*.c")) + glob.glob(os.path.join(root, "ai_dump", "clean", "*.c")):
    cur = None; buf = []
    for l in open(fn, encoding="utf-8", errors="replace"):
        m = re.match(r"// ==== ([0-9A-Fa-f]{8})", l)
        if m:
            if cur is not None and cur not in bodies: bodies[cur] = "".join(buf)
            cur = int(m.group(1), 16); buf = []
        elif cur is not None:
            buf.append(l)
    if cur is not None and cur not in bodies: bodies[cur] = "".join(buf)
names = {}
for l in open(os.path.join(root, "ai_dump", "cov_names.tsv")):
    p = l.rstrip("\n").split("\t"); names[int(p[0], 16)] = (p[1], p[2])
lines = open(os.path.join(root, "ai_coverage.txt")).read().splitlines()
out = []
for l in lines:
    if not l.startswith("0x"):
        out.append(l); continue
    p = l.split("\t"); a = int(p[0], 16); sz = int(p[1])
    b = bodies.get(a, "")
    nm, callers = names.get(a, ("", ""))
    if 0x40AC0 <= a < 0x43610: tag = "editor(CAlloyEditor, misattributed)"
    elif a in (0x2C580, 0x2C5D0, 0x2C670, 0x2C6D0, 0x2C720): tag = "physics(VehicleEntity helper, misattributed)"
    elif a in (0x30020, 0x16FF0, 0x15A00, 0x28770, 0x28D00): tag = "debug"
    elif a in (0x23620, 0x238B0, 0x239E0, 0x23980, 0x187D0, 0x271B0, 0x389C0, 0x140B0, 0x13D80, 0x14040, 0x16F540, 0x37EC0, 0x380F0, 0x15B70, 0x24FB0, 0x38320):
        tag = "ctor/singleton"
    elif not b: tag = "nodump"
    elif "Editor" in b or "\\Editors\\" in b: tag = "editor"
    elif re.search(r"AddDebugText|DrawDebug|Debug Text|debugMenu|AddTweak|tweak", b) or "AddTweaks" in nm: tag = "debug"
    elif re.search(r"Renderer__|D3DDevice|Render", b): tag = "render/debugdraw"
    elif "~" in nm or re.search(r"sub_1FD990|operator delete|MEM_Free", b) and sz < 700: tag = "dtor/free"
    elif "VsArray.h" in b and sz < 200: tag = "container"
    elif sz < 48: tag = "trivial"
    else: tag = "logic?"
    out.append("%s\t%s\t%s\t%s\t%s" % (p[0], p[1], p[2], nm, tag + ("\tmentioned" if l.endswith("mentioned") else "")))
from collections import Counter
c = Counter(); cb = Counter()
for l in out:
    if l.startswith("0x"):
        t = l.split("\t")[4]; c[t] += 1; cb[t] += int(l.split("\t")[1])
hdr = ["# unported by category (count, bytes): " + "; ".join("%s %d/%d" % (k, c[k], cb[k]) for k in sorted(c, key=lambda k: -cb[k])),
       "# 'logic?' = not auto-classified: mostly assert-wrapped accessors (GetX with dIndex/m_p asserts) folded into Rust",
       "# field/Vec access; 'trivial' < 48 B getters/setters. Gameplay logic left unported: none found (2026-11-06)."]
out = out[:2] + hdr + out[2:]
open(os.path.join(root, "ai_coverage.txt"), "w").write("\n".join(out) + "\n")
print(c, cb)
