# physics agent stage 4b: every call target inside the functions of research/phys_s4b_list.txt -> research/phys_s4b_calls.txt
import idc, idautils, ida_funcs
R = r"D:\projects\project poland rewrite\research"
out = []
for a in [int(x, 16) for x in open(R + r"\phys_s4b_list.txt").read().split()]:
    out.append("== %X" % a)
    for ea in idautils.FuncItems(a):
        if idc.print_insn_mnem(ea) == "call":
            t = idc.get_operand_value(ea, 0)
            out.append("  %X -> %X %s" % (ea, t, idc.get_func_name(t)))
open(R + r"\phys_s4b_calls.txt", "w").write("\n".join(out))
