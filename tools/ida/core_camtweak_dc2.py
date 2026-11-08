import ida_hexrays, idc, idautils
out = []
for ea in [0xC0A60]:
    out.append("// ==== %X\n%s" % (ea, str(ida_hexrays.decompile(ea))))
    for x in idautils.XrefsTo(ea):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.append("// caller %X in %X %s" % (x.frm, f, idc.get_func_name(f)))
        c = str(ida_hexrays.decompile(f))
        out.append(c)
open(r"D:/projects/project poland rewrite/research/core_camtweak_dc2.c", "w").write("\n".join(out))
print("ok")
