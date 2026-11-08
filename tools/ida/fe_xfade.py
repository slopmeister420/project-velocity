import ida_hexrays, ida_funcs
out = []
for ea in (0x00285350, 0x00285770, 0x00299D00, 0x002527A0, 0x00252830):
    f = ida_funcs.get_func(ea)
    try:
        out.append("// ===== %08X\n" % ea + str(ida_hexrays.decompile(f.start_ea)))
    except Exception as e:
        out.append("// ===== %08X fail %s" % (ea, e))
open(r"D:/projects/project poland rewrite/research/fe_xfade.c", "w").write("\n".join(out))
