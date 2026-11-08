import ida_hexrays, idc, idautils
out = []
for ea in [0x602D0, 0x5FFE0, 0x66830, 0x668C0]:
    try:
        out.append("// ==== %X %s\n%s" % (ea, idc.get_func_name(ea), str(ida_hexrays.decompile(ea))))
    except Exception as e:
        out.append("// ==== %X fail %s" % (ea, e))
for a in [0x74B4A8]:
    for x in idautils.XrefsTo(a):
        out.append("// xref %X %X %s %s" % (a, x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
open(r"D:/projects/project poland rewrite/research/core_camtweak_dc4.c", "w").write("\n".join(out))
print("ok")
