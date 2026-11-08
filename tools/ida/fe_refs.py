# frontend agent: all refs (code + data) to the given addresses
import idautils, idc
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    ea = int(a, 16)
    for x in idautils.XrefsTo(ea):
        print("%08X -> %s  %s  %s" % (x.frm, a, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
