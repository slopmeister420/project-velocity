# physics agent: dump CarDamage.cpp / CarDamageFX.cpp (0x6AFA0-0x79C90) decompilations and outside callers
import os, idc, idautils, ida_hexrays, ida_name, ida_funcs

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
os.makedirs(ROOT, exist_ok=True)
LO, HI = 0x6AFA0, 0x79C90
out = open(os.path.join(ROOT, "CarDamage.c"), "w", encoding="utf-8")
calls = open(os.path.join(ROOT, "callers.txt"), "w", encoding="utf-8")
n = 0
for fa in idautils.Functions(LO, HI):
    name = ida_name.get_name(fa)
    try:
        body = str(ida_hexrays.decompile(fa))
    except Exception as ex:
        body = "// fail %s" % ex
    out.write("// ==== %08X %s\n%s\n" % (fa, name, body))
    n += 1
    for x in idautils.XrefsTo(fa):
        f = ida_funcs.get_func(x.frm)
        if f and not (LO <= f.start_ea < HI):
            calls.write("%08X %s <- %08X %s\n" % (fa, name, f.start_ea, ida_name.get_name(f.start_ea)))
        elif not f:
            calls.write("%08X %s <- data %08X\n" % (fa, name, x.frm))
out.close(); calls.close()
print("dumped", n)
