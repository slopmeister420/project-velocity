# frontend agent (tips stage): call targets in CreateAllScreens -> research/fe_tips_calls.txt
import idc, idautils
with open(r"D:\projects\project poland rewrite\research\fe_tips_calls.txt", "w") as o:
    for ea in idautils.FuncItems(0x001FEB90):
        if idc.print_insn_mnem(ea) == "call":
            t = idc.get_operand_value(ea, 0)
            n = idc.get_func_name(t)
            if "Create" in n or "__2" in n:
                o.write("%08X %s\n" % (ea, n))
