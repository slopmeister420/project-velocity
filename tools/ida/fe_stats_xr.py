# frontend agent: callers (function + call site) of the stats list functions -> research/fe_stats_xr.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_stats_xr.txt", "w")
for a in "002894A0 002894E0 00289510 00289550 00289660 00289780 002897A0 0028A480 0028ACF0 0028AD40 0028AEC0 0028B110 0028B3A0".split():
    ea = int(a, 16)
    out.write("== %s %s\n" % (a, idc.get_func_name(ea)))
    for x in idautils.CodeRefsTo(ea, 0):
        out.write("   %08X %s\n" % (x, idc.get_func_name(x)))
out.close()
