# frontend agent: xrefs to addresses listed in research/fe_x_args.txt -> research/fe_x.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_x.txt", "w")
for a in [int(x,16) for x in open(r"D:\projects\project poland rewrite\research\fe_x_args.txt").read().split()]:
    out.write("== %08X\n" % a)
    for x in idautils.XrefsTo(a):
        out.write("  %08X %s | %s\n" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
        for y in idautils.XrefsTo(idc.get_func_attr(x.frm, idc.FUNCATTR_START)):
            out.write("      caller %08X %s\n" % (y.frm, idc.get_func_name(y.frm)))
out.close()
