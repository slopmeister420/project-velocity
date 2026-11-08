import re, os, sys
root = r"D:\projects\project poland rewrite"
KEYS = ("\\Feck2\\", "\\Osd\\", "OSDMap\\", "GameMenuManager\\", "FontHelper\\", "StringManager\\", "Fonts\\", "VsMovie\\")
regs = []
for l in open(os.path.join(root, "research", "objregions.tsv")):
    p = l.rstrip("\r\n").split("\t")
    if len(p) < 3: continue
    f = p[2]
    if any(k in f for k in KEYS) or f.startswith(("Fonts", "StringManager", "VsMovie")):
        regs.append((int(p[0], 16), int(p[1], 16), f))
funcs = []
for l in open(os.path.join(root, "research", "funcmap.tsv")):
    p = l.rstrip("\r\n").split("\t")
    a = int(p[0], 16); sz = int(p[1])
    for s, e, f in regs:
        if s <= a < e:
            funcs.append((a, sz, f))
orig = set(); anyhex = set()
for dp, dn, fn in os.walk(os.path.join(root, "crates", "velocity_frontend", "src")):
    for n in fn:
        if not n.endswith(".rs"): continue
        for l in open(os.path.join(dp, n), encoding="utf-8", errors="replace"):
            s = l.strip()
            if s.startswith("//") and re.search(r"\w\.cpp\s+0x[0-9A-Fa-f]+", s):
                # origin line: `/// File.cpp 0xA / 0xB / Other.cpp 0xC — text` (merged functions after the first)
                m = re.match(r"///?\s*\S+\.cpp\s+(0x[0-9A-Fa-f]+(\s*[/,+]\s*(\S+\.cpp\s+)?0x[0-9A-Fa-f]+)*)", s)
                if m:
                    for h in re.findall(r"0x([0-9A-Fa-f]+)", m.group(1)): orig.add(int(h, 16))
            for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", s):
                anyhex.add(int(h, 16))
tot = len(funcs); totb = sum(f[1] for f in funcs)
port = [f for f in funcs if f[0] in orig]
miss = [f for f in funcs if f[0] not in orig]
with open(os.path.join(root, "research", "fe_cov_raw.tsv"), "w") as o:
    for a, sz, f in sorted(miss, key=lambda x: -x[1]):
        o.write("%08X\t%d\t%s\t%d\n" % (a, sz, f.split("\\")[-1], a in anyhex))
print("ported %d/%d funcs, %d/%d bytes (%.1f%%)" % (len(port), tot, sum(f[1] for f in port), totb, 100.0 * sum(f[1] for f in port) / totb))
