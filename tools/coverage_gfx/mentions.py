"""dump, for every 'mentioned' function in research/graphics_coverage.txt, where its address appears in visos_render.
usage: python tools/coverage_gfx/mentions.py -> research/gfx_mentions.tsv
cols: addr size file tag | rs-file:line | kind (doc = in a /// block, line = // comment, code) | next fn | text"""
import re, os
root = r"D:\projects\project poland rewrite"
men = {}
for l in open(os.path.join(root, "research", "graphics_coverage.txt")):
    if l.startswith("#"): continue
    p = l.rstrip("\n").split("\t")
    if len(p) >= 6 and p[5] == "mentioned":
        men[int(p[0], 16)] = (p[1], p[2], p[4])
out = []
for dp, dn, fn in os.walk(os.path.join(root, "crates", "visos_render", "src")):
    for n in fn:
        if not n.endswith(".rs"): continue
        path = os.path.join(dp, n)
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
        for i, l in enumerate(lines):
            for h in re.findall(r"0x([0-9A-Fa-f]{5,8})\b", l):
                a = int(h, 16)
                if a not in men: continue
                s = l.strip()
                kind = "doc" if s.startswith("///") else ("line" if s.startswith("//") else ("trail" if "//" in l else "code"))
                nxt = ""
                if kind == "doc":
                    b = i
                    while b > 0 and lines[b - 1].strip().startswith("///"): b -= 1
                    lead = re.match(r"///\s*(\S+\.cpp\s+|[A-Za-z_:]+\s+)?(0x[0-9A-Fa-f]+(\s*[/,]\s*0x[0-9A-Fa-f]+)*)", s)
                    own = lead and ("%X" % a) in lead.group(2).upper().replace("0X", " ").split() or False
                    if not own and lead:
                        own = any(int(x, 16) == a for x in re.findall(r"0x([0-9A-Fa-f]+)", lead.group(2)))
                    kind = ("doc-own-first" if b == i else "doc-own") if own else ("doc-first" if b == i else "doc")
                    for j in range(i + 1, min(i + 40, len(lines))):
                        t = lines[j].strip()
                        if t.startswith("///") or t.startswith("#["): continue
                        m = re.search(r"\b(fn|struct|enum|impl|const|static|type|mod|trait)\s+(\w+)", t)
                        nxt = (m.group(1) + " " + m.group(2)) if m else t[:30]
                        break
                rel = os.path.relpath(path, os.path.join(root, "crates", "visos_render", "src"))
                sz, f, tg = men[a]
                out.append("0x%06X\t%s\t%s\t%s\t%s:%d\t%s\t%s\t%s" % (a, sz, f, tg, rel, i + 1, kind, nxt, s[:160]))
out.sort()
open(os.path.join(root, "research", "gfx_mentions.tsv"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(men), len(out), len(set(o.split("\t")[0] for o in out)))
