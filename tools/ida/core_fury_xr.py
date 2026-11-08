import idc, idautils, ida_hexrays
# xrefs of the fury / zone time stretch globals and 0x00128680 callers -> research/core_fury_xr.txt
out = open(r"D:\projects\project poland rewrite\research\core_fury_xr.txt", "w")
for ea in (0x6F5B70, 0x686B5C, 0x128680, 0x154FC0):
    out.write("== %x %s\n" % (ea, idc.get_name(ea)))
    for x in idautils.XrefsTo(ea):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("  %x type %d in %x %s : %s\n" % (x.frm, x.type, f if f != idc.BADADDR else 0, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
out.close()
