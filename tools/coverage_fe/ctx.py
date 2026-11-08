"""for every unported-but-mentioned frontend function (research/fe_cov_raw.tsv col 4 = 1): where the address is
mentioned in crates/velocity_frontend/src, the enclosing rust fn and its origin line, and a verdict hint.
writes research/fe_cov_mentions.tsv: addr size name | file:line | fn | origin | hint | text"""
import os, re
root = r"D:\projects\project poland rewrite"
src = os.path.join(root, "crates", "velocity_frontend", "src")
raw = []
for l in open(os.path.join(root, "research", "fe_cov_raw.tsv")):
    p = l.rstrip("\n").split("\t")
    if p[3] == "1": raw.append((int(p[0], 16), int(p[1]), p[2]))
names = {}
for l in open(os.path.join(root, "research", "fe_cov_names.tsv")):
    p = l.rstrip("\n").split("\t")
    if len(p) > 1: names[p[0]] = p[1]
files = {}
for dp, dn, fn in os.walk(src):
    for n in fn:
        if n.endswith(".rs"):
            path = os.path.join(dp, n)
            files[os.path.relpath(path, src)] = open(path, encoding="utf-8", errors="replace").read().splitlines()
FN = re.compile(r"^\s*(pub(\(\w+\))?\s+)?(const\s+)?(unsafe\s+)?fn\s+(\w+)")
ORIG = re.compile(r"^\s*///\s*\S+\.cpp\s+0x")
NEG = re.compile(r"not ported|STAND-IN|not drawn|skipped|unported|not wired|dropped|TODO|not yet", re.I)
def enclosing(lines, i):
    # doc comment above a fn: look down; else look up for the fn whose body holds line i
    j = i
    while j < len(lines) and lines[j].strip().startswith(("///", "#[")): j += 1
    if j < len(lines) and FN.match(lines[j]) and lines[i].strip().startswith("///"):
        k = j
    else:
        k = i
        while k >= 0 and not FN.match(lines[k]): k -= 1
        if k < 0: return "-", "-"
    o = k - 1
    origin = "-"
    while o >= 0 and lines[o].strip().startswith(("///", "#[")):
        if ORIG.match(lines[o]): origin = lines[o].strip()
        o -= 1
    return FN.match(lines[k]).group(5) + ":%d" % (k + 1), origin
out = []
for a, sz, f in raw:
    pat = re.compile(r"0x0*%X\b" % a, re.I)
    hits = []
    for rel, lines in files.items():
        for i, l in enumerate(lines):
            if pat.search(l): hits.append((rel, i, l))
    for rel, i, l in hits[:3]:
        fn, origin = enclosing(files[rel], i)
        hint = "neg" if NEG.search(l) else ""
        out.append("0x%06X\t%d\t%s\t%s\t%s:%d\t%s\t%s\t%s\t%s" % (a, sz, f, names.get("%08X" % a, ""), rel, i + 1, fn,
                                                         origin[:70], hint, l.strip()[:160]))
open(os.path.join(root, "research", "fe_cov_mentions.tsv"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(raw), len(out))
