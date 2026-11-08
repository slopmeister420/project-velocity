# physics agent (handling review): writes of displacement 0x444 (race vehicle +1092 model) in the AI range
import idautils, idc, ida_ua
DS = (0x444,)
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        op = insn.ops[0]
        if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) in DS and idc.print_insn_mnem(ea) == "mov":
            out.append("%X %s %s" % (ea, idc.get_func_name(f), idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\phys_hr_disp.txt", "w").write("\n".join(out))
