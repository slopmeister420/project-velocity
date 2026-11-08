# frontend agent: all xrefs (code + data) to addresses in fe_peek_args.txt -> research/fe_xrefs.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_xrefs.txt", "w")
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    ea = int(a, 16)
    out.write("## %08X %s\n" % (ea, idc.get_func_name(ea)))
    for x in idautils.XrefsTo(ea, 0):
        out.write("  %08X type %d in %s\n" % (x.frm, x.type, idc.get_func_name(x.frm)))
        for y in idautils.XrefsTo(x.frm, 0):
            out.write("    <- %08X in %s\n" % (y.frm, idc.get_func_name(y.frm)))
out.close()
