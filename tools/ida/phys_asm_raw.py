# physics agent: like phys_asm_range.py but with numeric operand values (stack displacements)
import idc
R = r"D:\projects\project poland rewrite\research"
s, e = [int(x, 16) for x in open(R + r"\_asm_range.txt").read().split()]
out = []
ea = s
while ea < e:
    ops = " | ".join("%s=%#x" % (idc.print_operand(ea, k), idc.get_operand_value(ea, k)) for k in range(2) if idc.get_operand_type(ea, k))
    out.append("%08X  %s  [%s]" % (ea, idc.print_insn_mnem(ea), ops))
    ea = idc.next_head(ea, e + 16)
open(R + r"\_asm.txt", "w").write("\n".join(out))
