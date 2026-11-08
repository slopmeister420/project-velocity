# frontend agent: list code refs to the given data addresses with the instruction (fe_peek_args.txt: addr...)
import idautils, idc
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    ea = int(a, 16)
    print("==", hex(ea))
    for x in idautils.XrefsTo(ea):
        print("  %08X %-30s %s" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
