# physics agent: decompile the outside callers of CarDamage into research/damage_dump/Callers.c
import os, ida_hexrays, ida_name, ida_funcs

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
ADDRS = [int(a, 16) for a in open(os.path.join(ROOT, "addrs.txt")).read().split()]
with open(os.path.join(ROOT, "Callers.c"), "w", encoding="utf-8") as o:
    for a in ADDRS:
        f = ida_funcs.get_func(a)
        try:
            b = str(ida_hexrays.decompile(f.start_ea))
        except Exception as ex:
            b = "// fail %s" % ex
        o.write("// ==== %08X %s\n%s\n" % (f.start_ea, ida_name.get_name(f.start_ea), b))
print("ok", len(ADDRS))
