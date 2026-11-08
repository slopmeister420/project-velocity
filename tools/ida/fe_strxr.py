# frontend agent: functions referencing the string addresses in fe_strxr_args.txt -> research/fe_strxr.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_strxr.txt", "w", encoding="utf-8")
for t in open(r"D:\projects\project poland rewrite\tools\ida\fe_strxr_args.txt").read().split():
    ea = int(t, 16)
    out.write("== %08X %s\n" % (ea, idc.get_strlit_contents(ea, -1, 0)))
    for x in idautils.XrefsTo(ea, 0):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("  %08X in %08X %s\n" % (x.frm, f if f != idc.BADADDR else 0, idc.get_func_name(x.frm)))
out.close()
