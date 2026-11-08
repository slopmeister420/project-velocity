# dump decompilation of the ai object regions to research/ai_dump/<file>.c (ai agent)
import os, idautils, idc, ida_hexrays, ida_funcs, ida_name

OUT = r"D:\projects\project poland rewrite\research\ai_dump"
REGIONS = []
for line in open(r"D:\projects\project poland rewrite\research\objregions.tsv"):
    s, e, f = line.rstrip("\n").split("\t")
    if ("\\Ai\\" in f) or ("OpponentDatabase" in f) or ("TeamDatabase" in f) or ("TimelineAI" in f) or ("TrafficBinLoading" in f):
        REGIONS.append((int(s, 16), int(e, 16), f))
ONLY = os.environ.get("AI_DUMP_ONLY")
os.makedirs(OUT, exist_ok=True)
for s, e, f in REGIONS:
    stem = f.split("\\")[-1].replace(".cpp", "")
    if ONLY and stem != ONLY:
        continue
    with open(os.path.join(OUT, stem + ".c"), "w", encoding="utf-8") as o:
        for fea in idautils.Functions(s, e):
            name = ida_name.get_name(fea)
            o.write("\n// ==== %08X %s\n" % (fea, name))
            try:
                o.write(str(ida_hexrays.decompile(fea, flags=ida_hexrays.DECOMP_NO_CACHE)))
            except Exception as ex:
                o.write("// decompile failed: %s\n" % ex)
    print(stem, "done")
