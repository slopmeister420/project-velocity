import ida_hexrays, ida_funcs
out = []
for ea in (0x0024D260, 0x002528D0, 0x0026FC40, 0x0010CE80):
    f = ida_funcs.get_func(ea)
    try:
        out.append("// ===== %08X\n" % ea + str(ida_hexrays.decompile(f.start_ea)))
    except Exception as e:
        out.append("// ===== %08X fail %s" % (ea, e))
open(r"D:/projects/project poland rewrite/research/fe_7065a4.c", "w").write("\n".join(out))
