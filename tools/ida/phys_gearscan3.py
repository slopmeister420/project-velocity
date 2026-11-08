# physics agent: accesses of the manual shift flags VE+0xDC9/0xDCA (car+0xCD9/0xCDA) in all game code
import idautils, idc, ida_ua
DS = (0xDC8, 0xDC9, 0xDCA, 0xCD8, 0xCD9, 0xCDA)
out = []
for f in idautils.Functions(0x11000, 0x3C63D0):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and (op.addr & 0xFFFFFFFF) in DS:
                out.append("%X %s %s" % (ea, idc.get_func_name(f), idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\phys_gearscan3.txt", "w").write("\n".join(out))
print(len(out))
