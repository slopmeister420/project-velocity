import idautils, idc, ida_hexrays, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/core_savetable2.c", "w")
for f in [0x2FFD00]:
    out.write("\n// ===== %X %s\n" % (f, idc.get_func_name(f)))
    out.write(str(ida_hexrays.decompile(f)))
for t in [0x2FFD30, 0x2FFD00, 0x300680, 0x304ED0, 0x2FE790]:
    out.write("\n// xrefs %X\n" % t)
    for x in idautils.XrefsTo(t):
        out.write("//   %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
out.close()
