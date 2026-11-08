# physics agent (handling review): disassembly of one function -> research/phys_hr_asm2.txt
import idautils, idc
EA = 0x372B70
out = []
for i in idautils.FuncItems(EA):
    out.append("%X %s" % (i, idc.GetDisasm(i)))
open(r"D:\projects\project poland rewrite\research\phys_hr_asm2.txt", "w").write("\n".join(out))
