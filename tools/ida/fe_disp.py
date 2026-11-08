# frontend agent: instructions with a [reg+disp] operand equal to any displacement in fe_disp_args.txt -> research/fe_disp.txt
import idautils, idc
want = set(int(t, 0) for t in open(r"D:\projects\project poland rewrite\tools\ida\fe_disp_args.txt").read().split())
out = open(r"D:\projects\project poland rewrite\research\fe_disp.txt", "w")
for ea in idautils.Heads(0x11000, 0x3DDD20):
    for n in (0, 1):
        if idc.get_operand_type(ea, n) == 4 and idc.get_operand_value(ea, n) in want:
            out.write("%08X %-40s %s\n" % (ea, idc.get_func_name(ea), idc.GetDisasm(ea)))
            break
out.close()
print("ok")
