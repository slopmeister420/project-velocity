"""from research/gfx_mentions.tsv: one row per doc line whose leading address list cites graphics functions it ports.
-> research/gfx_origin_plan.tsv: rs-file:line \t old lead \t new lead \t note"""
import re, os
root = r"D:\projects\project poland rewrite"
fileof = {}
for l in open(os.path.join(root, "research", "graphics_coverage.txt")):
    if l.startswith("#"): continue
    p = l.rstrip("\n").split("\t")
    fileof[int(p[0], 16)] = p[2]
# judged by hand: lead word is prose ("init"), or a second copy of a helper already cited elsewhere
SKIP = {0x147DC0}
SKIPLOC = {"car_damage_fx.rs:83", "light_streak.rs:42", "car_shader_manager_pc_xbox.rs:111"}
rows = {}
for l in open(os.path.join(root, "research", "gfx_mentions.tsv"), encoding="utf-8"):
    p = l.rstrip("\n").split("\t")
    if p[5] != "doc-own-first" or not p[6].startswith("fn "): continue
    if int(p[0], 16) in SKIP or p[4] in SKIPLOC: continue
    rows[p[4]] = p[7]
out = []
for loc, s in sorted(rows.items()):
    m = re.match(r"///\s*(\S+\.cpp\s+|[A-Za-z_:]+\s+)?(0x[0-9A-Fa-f]+(\s*[/,]\s*0x[0-9A-Fa-f]+)*)", s)
    addrs = [int(x, 16) for x in re.findall(r"0x([0-9A-Fa-f]+)", m.group(2))]
    files = sorted(set(fileof[a] for a in addrs if a in fileof))
    word = (m.group(1) or "").strip()
    note = "" if len(files) == 1 else "MULTI " + ",".join(files)
    f = files[0] if files else "?"
    stem = f[:-4]
    keep = (" " + word) if word and word.split("::")[0] != stem else ""
    old = m.group(0)
    new = "/// %s %s%s" % (f, m.group(2), keep)
    out.append("%s\t%s\t%s\t%s" % (loc, old, new, note))
open(os.path.join(root, "research", "gfx_origin_plan.tsv"), "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(out), sum(1 for o in out if "MULTI" in o))
