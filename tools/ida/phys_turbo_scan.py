# physics agent: stores to [reg+94h] / [reg+90h] (VE turbo ratio / flag) in the vehicle code -> research/phys_dump/turbo_scan.txt
import idc, ida_ua, idautils, idaapi

OUT = r"D:\projects\project poland rewrite\research\phys_dump\turbo_scan.txt"
with open(OUT, "w") as o:
    for fea in idautils.Functions(0x001B3000, 0x001E0000):
        for a in idautils.FuncItems(fea):
            insn = ida_ua.insn_t()
            if ida_ua.decode_insn(insn, a) <= 0:
                continue
            op = insn.ops[0]
            if op.type == idaapi.o_displ and op.addr in (0x90, 0x94):
                m = idc.print_insn_mnem(a)
                if m.startswith("mov") or m.startswith("fst"):
                    o.write("%08X %-28s %s\n" % (a, idc.get_func_name(a), idc.generate_disasm_line(a, 0)))
print("ok")
