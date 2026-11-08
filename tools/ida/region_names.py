# name sub_ functions in the game range by the object file region they fall in
import idautils, idc, ida_funcs, ida_name, re
ROOT = r"D:\projects\project poland rewrite\research"
GAME_END = 0x3DDD20
starts = []
for line in open(ROOT + r"\fileranges.txt"):
    a, b, n, f = line.split(None, 3)
    starts.append((int(a, 16), f.strip()))
starts.sort()
rows = []
renamed = 0
for i, (a, f) in enumerate(starts):
    end = starts[i + 1][0] if i + 1 < len(starts) else GAME_END
    if a >= GAME_END or "xbox-dec03" in f:
        continue
    stem = re.sub(r"\W", "_", f.replace("\\", "/").split("/")[-1].rsplit(".", 1)[0])
    rows.append("%08x\t%08x\t%s" % (a, end, f))
    for fea in idautils.Functions(a, end):
        n = idc.get_func_name(fea)
        if n.startswith("sub_"):
            if ida_name.set_name(fea, "%s__%X" % (stem, fea), ida_name.SN_NOWARN | ida_name.SN_NOCHECK):
                renamed += 1
open(ROOT + r"\objregions.tsv", "w").write("\n".join(rows))
print("regions", len(rows), "renamed", renamed)
