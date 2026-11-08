# frontend agent: functions in the results OSD object range and their callers -> research/fe_res_funcs.txt
import idautils, idc
with open(r"D:\projects\project poland rewrite\research\fe_res_funcs.txt", "w") as o:
    for f in idautils.Functions(0xF2E00, 0xF3A70):
        callers = sorted(set(idc.get_func_name(x) or hex(x) for x in idautils.CodeRefsTo(f, 0)))
        o.write("%08X %s <- %s\n" % (f, idc.get_func_name(f), ", ".join(callers)))
