# frontend agent: data xrefs to the addresses in fe_dxref_args.txt -> research/fe_dxref.txt
import idautils, idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_dxref_args.txt").read().split()
with open(r"D:\projects\project poland rewrite\research\fe_dxref.txt", "w") as o:
    for t in a:
        ea = int(t, 16)
        for x in idautils.DataRefsTo(ea):
            o.write("%08X <- %08X %s\n" % (ea, x, idc.get_func_name(x) or idc.get_name(x)))
        for x in idautils.CodeRefsTo(ea, 0):
            o.write("%08X <c %08X %s\n" % (ea, x, idc.get_func_name(x)))
