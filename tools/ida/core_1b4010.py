import idautils, idc, ida_hexrays
out = open(r"D:/projects/project poland rewrite/research/core_1b4010.c", "w")
for f in [0x1B4010]:
    out.write("\n// ===== %X %s\n" % (f, idc.get_func_name(f)))
    out.write(str(ida_hexrays.decompile(f)))
    for x in idautils.XrefsTo(f):
        out.write("//   xref %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
out.close()
