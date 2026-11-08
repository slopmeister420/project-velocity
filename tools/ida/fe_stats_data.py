# frontend agent: data for the R2 stats list -> research/fe_stats_data.txt
import idc, idautils, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\fe_stats_data.txt", "w")
for name, ea in (("cat 687850", 0x687850), ("sub 6878E0", 0x6878E0)):
    out.write("%s: %s\n" % (name, " ".join(str(idc.get_wide_dword(ea + 4 * i)) for i in range(35))))
out.write("77FD9C xrefs:\n")
for x in idautils.DataRefsTo(0x77FD9C):
    out.write("  %08X %s  %s\n" % (x, idc.get_func_name(x), idc.GetDisasm(x)))
for ea in (0x2105B0,):
    out.write(str(ida_hexrays.decompile(ea)) + "\n")
out.close()
