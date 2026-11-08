# physics agent (handling review): disassembly of named functions -> research/phys_hr_asm.txt
import idautils, idc, ida_name, idaapi
NAMES = ["VsSqrt", "VsFabs"]
out = []
for n in NAMES:
    ea = ida_name.get_name_ea(idaapi.BADADDR, n)
    out.append("== %s %X" % (n, ea))
    if ea == idaapi.BADADDR:
        continue
    for i in idautils.FuncItems(ea):
        out.append("%X %s" % (i, idc.GetDisasm(i)))
open(r"D:\projects\project poland rewrite\research\phys_hr_asm.txt", "w").write("\n".join(out))
