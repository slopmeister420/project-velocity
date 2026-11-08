import idautils, idc, ida_xref
out = []
for g in (0x7691B8, 0x768978, 0x768C30, 0x7696B0):
    for x in idautils.XrefsTo(g, 0):
        if x.type == ida_xref.dr_W:
            out.append("%08X <- %08X %s | %s" % (g, x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
open(r"D:\projects\project poland rewrite\research\fe_global_writers.txt", "w").write("\n".join(out) + "\n")
