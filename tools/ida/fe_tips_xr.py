# frontend agent (tips stage): code refs to the TIPS\ / CUTS\ strings with surrounding disassembly -> research/fe_tips_xr.txt
import idc, idautils
with open(r"D:\projects\project poland rewrite\research\fe_tips_xr.txt", "w") as o:
    for s in (0x0065613C, 0x00654200):
        for x in idautils.DataRefsTo(s):
            o.write("== %08X <- %08X %s\n" % (s, x, idc.get_func_name(x)))
            ea = idc.prev_head(idc.prev_head(x))
            for _ in range(8):
                o.write("%08X %s\n" % (ea, idc.GetDisasm(ea)))
                ea = idc.next_head(ea)
