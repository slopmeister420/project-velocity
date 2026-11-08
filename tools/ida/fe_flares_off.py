import ida_hexrays, ida_funcs, idautils
out = []
for ea in (0x0021EF50,):
    out.append("// ===== %08X\n" % ea + str(ida_hexrays.decompile(ea)))
    out.append("// callers: " + " ".join("%X" % x for x in idautils.CodeRefsTo(ea, 0)) + " data: " + " ".join("%X" % x for x in idautils.DataRefsTo(ea)))
open(r"D:/projects/project poland rewrite/research/fe_flares_off.c", "w").write("\n".join(out))
