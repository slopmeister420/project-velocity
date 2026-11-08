# physics agent: dump decompilation of physics regions to research/phys_dump/<file>.c
# PHYS_DUMP_ONLY=<stem> limits to one file
import os, idautils, idc, ida_hexrays, ida_name

ROOT = r"D:\projects\project poland rewrite\research"
OUT = os.path.join(ROOT, "phys_dump")
KEYS = ("\\Physics\\", "\\CarDamage\\", "\\VehicleDatabase\\")
REGIONS = []
for line in open(os.path.join(ROOT, "objregions.tsv")):
    s, e, f = line.rstrip("\n").split("\t")
    if any(k in f for k in KEYS) and "TimelineManager" not in f:
        REGIONS.append((int(s, 16), int(e, 16), f))
only = None
try:
    only = open(os.path.join(ROOT, "phys_dump_only.txt")).read().strip() or None
except Exception:
    pass
os.makedirs(OUT, exist_ok=True)
for s, e, f in REGIONS:
    stem = f.split("\\")[-1].replace(".cpp", "")
    if only and stem not in only.split(","):
        continue
    n = 0
    with open(os.path.join(OUT, stem + ".c"), "w", encoding="utf-8") as o:
        for fea in idautils.Functions(s, e):
            n += 1
            o.write("\n// ==== %08X %s\n" % (fea, ida_name.get_name(fea)))
            try:
                o.write(str(ida_hexrays.decompile(fea)))
            except Exception as ex:
                o.write("// decompile failed: %s\n" % ex)
    print(stem, n)
