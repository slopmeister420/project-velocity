# physics agent: stores to [reg+30h] in the vehicle code (VE+48 manual-gear hold timer writers)
import idautils, idc, ida_ua
out = []
for f in idautils.Functions(0x1B4000, 0x1E0000):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        op = insn.ops[0]
        if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) == 0x30:
            m = idc.print_insn_mnem(ea)
            if m.startswith("mov") or m.startswith("fst"):
                out.append("%X %s %s" % (ea, idc.get_func_name(f), idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\phys_gearscan2.txt", "w").write("\n".join(out))
print(len(out))
