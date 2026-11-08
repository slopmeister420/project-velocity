# frontend agent: xrefs to a global -> research/fe_xr724c.txt (address in the loop below)
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_xr724c.txt", "w", encoding="utf-8")
for g in (0x6F9058,):
    for x in idautils.XrefsTo(g):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("%08X %d %s %s\n" % (x.frm, x.type, idc.get_func_name(f), idc.generate_disasm_line(x.frm, 0)))
out.close()
