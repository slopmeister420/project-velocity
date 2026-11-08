# xrefs to the TrailBlazerRender functions / globals -> research/gfx/tb_xref.txt
import idautils, idc, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
eas = [0x14F960, 0x150140, 0x1501E0, 0x151880, 0x151DC0, 0x151F50, 0x152100, 0x1521E0, 0x152290, 0x152380,
       0x1523B0, 0x7374C4, 0x7374C8, 0x7374CC, 0x68741C, 0x687408, 0x7187E0]
out = []
for ea in eas:
    out.append("== %X %s" % (ea, idc.get_name(ea)))
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s" % (x.frm, f and idc.get_func_name(f.start_ea)))
# functions in the region and just before/after
for f in idautils.Functions(0x14E000, 0x152400):
    out.append("fn %X %s" % (f, idc.get_func_name(f)))
open(B + r"\tb_xref.txt", "w").write("\n".join(out))
print("ok")
