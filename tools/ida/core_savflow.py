import idautils, idc, ida_hexrays, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/core_saveflow.c", "w")
for lo, hi in [(0x2FFD30, 0x300F30), (0x304D80, 0x305070), (0x30E3B0, 0x30F960)]:
    for f in idautils.Functions(lo, hi):
        out.write("\n// ===== %X %s\n" % (f, idc.get_func_name(f)))
        try:
            out.write(str(ida_hexrays.decompile(f)))
        except Exception as e:
            out.write("// decompile failed: %s\n" % e)
out.close()
