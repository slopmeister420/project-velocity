"""tag the unported entries of research/<sub>_coverage.txt (core): python tools/coverage/tag.py core
needs research/core_names.tsv (tools/ida/cov_names.py): addr name callers drefs flags
"""
import re, os, sys
from collections import Counter
root = r"D:\projects\project poland rewrite\research"
sub = sys.argv[1]
info = {}
for l in open(os.path.join(root, sub + "_names.tsv")):
    p = l.rstrip("\n").split("\t")
    info[int(p[0], 16)] = (p[1], int(p[2]), int(p[3]), p[4] if len(p) > 4 else "")
PLATFORM = ("MEM_", "XMem", "VsPoolAllocator", "VsStringAnsi", "VsFileDirectory", "_FILE_", "FILE_LoadAsync",
            "FILE_Init", "Performance", "HANDY_", "MEM_Munge", "FILE_VCDLoader", "InputManager",
            "InputBinding", "InputMapping", "InputDevice")
# optional research/<sub>_func_sig.tsv (tools/ida/core_cov_sig.py): addr size hash net ... for every scope function;
# splits "gameplay" into network (the cut Xbox Live / system link mode) and template duplicates (same masked
# instruction sequence as a ported function, or as a lower-addressed unported one)
sig = {}
sp = os.path.join(root, sub + "_func_sig.tsv")
if os.path.exists(sp):
    for l in open(sp, encoding="utf-8"):
        p = l.rstrip("\n").split("\t")
        if len(p) >= 4:
            sig[int(p[0], 16)] = (int(p[1]), p[2], p[3] == "1")
lines = open(os.path.join(root, sub + "_coverage.txt")).read().splitlines()
unported = {int(l.split("\t")[0], 16) for l in lines if l.startswith("0x")}
groups = {}
for a, (sz, h, net) in sig.items():
    if sz >= 48:
        groups.setdefault(h, []).append(a)


def is_dup(a):
    g = groups.get(sig[a][1], []) if a in sig else []
    if len(g) < 2:
        return False
    if any(x not in unported for x in g):
        return True
    return a != min(g)


out = []
for l in lines:
    if not l.startswith("0x"):
        if not l.startswith("# unported by"):
            out.append(l)
        continue
    p = l.split("\t")
    a = int(p[0], 16)
    sz = int(p[1])
    f = p[2]
    nm, callers, drefs, fl = info.get(a, ("", 0, 0, ""))
    stem = f[:-4]
    cls = nm.split("::")[0] if "::" in nm else ""
    if any(f.startswith(x) for x in PLATFORM):
        tag = "platform(replaced by Rust/Bevy)"
    elif cls and cls.lstrip("~") != stem and not stem.startswith(cls) and not cls.startswith(stem):
        tag = "mis-attributed(%s)" % cls
    elif "dbgstr" in fl or re.search(r"Debug|Tweak|DumpMap|Dump", nm) or f == "tweak.cpp":
        tag = "debug"
    elif "~" in nm or ("free" in fl and sz < 400 and not nm.startswith("Update")):
        tag = "ctor-dtor"
    elif "vtbl" in fl and sz < 600:
        tag = "ctor-dtor"
    elif re.search(r"::(Init|Create|Construct|Destroy|Shutdown)|Ctor", nm):
        tag = "ctor-dtor"
    elif sz < 48 or re.search(r"::(Get|Is|Set|Has)[A-Z]", nm):
        tag = "getter"
    elif a in sig and sig[a][2]:
        tag = "network(cut online mode)"
    elif is_dup(a):
        tag = "template-dup"
    else:
        tag = "gameplay"
    out.append("%s\t%s\t%s\t%s\t%s%s" % (p[0], p[1], f, nm, tag, "\tmentioned" if l.endswith("mentioned") else ""))
c = Counter()
cb = Counter()
for l in out:
    if l.startswith("0x"):
        t = l.split("\t")[4]
        c[t.split("(")[0]] += 1
        cb[t.split("(")[0]] += int(l.split("\t")[1])
hdr = "# unported by category (count/bytes): " + "; ".join("%s %d/%d" % (k, c[k], cb[k]) for k in sorted(cb, key=lambda k: -cb[k]))
out = out[:2] + [hdr] + out[2:]
open(os.path.join(root, sub + "_coverage.txt"), "w").write("\n".join(out) + "\n")
print(hdr)
