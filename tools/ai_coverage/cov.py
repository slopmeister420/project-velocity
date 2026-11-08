import re, os, sys
root = r"D:\projects\project poland rewrite"
regs = []
for l in open(os.path.join(root, "research", "objregions.tsv")):
    p = l.rstrip("\r\n").split("\t")
    if len(p) < 3: continue
    f = p[2]
    if ("\\Ai\\" in f) or "OpponentDatabase" in f or "TeamDatabase" in f:
        regs.append((int(p[0], 16), int(p[1], 16), f))
funcs = []
for l in open(os.path.join(root, "research", "funcmap.tsv")):
    p = l.rstrip("\r\n").split("\t")
    a = int(p[0], 16); sz = int(p[1])
    for s, e, f in regs:
        if s <= a < e:
            funcs.append((a, sz, f))
# origin addresses: /// lines with File.cpp 0x...
orig = set(); anyhex = set()
for dp, dn, fn in os.walk(os.path.join(root, "crates", "velocity_ai", "src")):
    for n in fn:
        if not n.endswith(".rs"): continue
        for l in open(os.path.join(dp, n), encoding="utf-8", errors="replace"):
            s = l.strip()
            if s.startswith("//") and re.search(r"\w\.cpp\s+0x[0-9A-Fa-f]+", s):
                for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s):
                    orig.add(int(h, 16))
                m = re.match(r"///?\s*\S+\.cpp\s+(0x[0-9A-Fa-f]+(\s*/\s*0x[0-9A-Fa-f]+)*)", s)
                if m:
                    for h in re.findall(r"0x([0-9A-Fa-f]+)", m.group(1)): orig.add(int(h, 16))
            for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s):
                anyhex.add(int(h, 16))
names = {}
if os.path.exists(sys.argv[1] if len(sys.argv) > 1 else ""):
    for l in open(sys.argv[1]):
        p = l.rstrip("\n").split("\t")
        if len(p) >= 2: names[int(p[0], 16)] = p[1]
tot = len(funcs); totb = sum(f[1] for f in funcs)
port = [f for f in funcs if f[0] in orig]
miss = [f for f in funcs if f[0] not in orig]
pb = sum(f[1] for f in port)
out = []
out.append("# ai coverage: functions in ai regions (objregions.tsv) vs '/// File.cpp 0x...' origin lines in crates/velocity_ai")
out.append("# ported %d/%d functions, %d/%d bytes (%.1f%%)" % (len(port), tot, pb, totb, 100.0 * pb / totb))
out.append("# unported (addr size file name [mentioned = address appears in a comment but has no origin line]), largest first")
for a, sz, f in sorted(miss, key=lambda x: -x[1]):
    out.append("0x%08X\t%d\t%s\t%s%s" % (a, sz, f.split("\\")[-1], names.get(a, ""), "\tmentioned" if a in anyhex else ""))
open(os.path.join(root, "research", "ai_coverage.txt"), "w").write("\n".join(out) + "\n")
print(out[1])
