# physics agent: GC / LC field name for byte offsets given on the command line (prefix L for LC)
import re, sys
src = open(r"D:\projects\project poland rewrite\crates\velocity_physics\src\c_config.rs", encoding="utf-8").read()
g = src[src.index("pub struct CGlobalConfig"):src.index("pub enum LoadResult")]
l = src[src.index("pub struct CLocalConfig"):]
def table(t):
    d = {}
    for m in re.finditer(r"pub (\w+): (.+?), // \+0x([0-9A-F]+)", t):
        n, ty, o = m.groups(); o = int(o, 16)
        mm = re.match(r"\[(?:f32|i32|Expression); (\d+)\]", ty.strip())
        cnt = int(mm.group(1)) if mm else 1
        if ty.strip().startswith("[[") : cnt = 24
        for k in range(cnt):
            d[o + 4 * k] = n + ("[%d]" % k if cnt > 1 else "")
    return d
G, L = table(g), table(l)
for a in sys.argv[1:]:
    if a.startswith("L"):
        o = int(a[1:]); print(a, L.get(o, "?"))
    else:
        o = int(a); print(a, G.get(o, "?"))
