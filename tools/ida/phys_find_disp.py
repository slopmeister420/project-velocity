# physics agent: instructions using given displacements (member search)
import idautils, idc, ida_ua, ida_funcs
DS = (0x1338, 0x133C)
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if ida_ua.decode_insn(insn, ea) == 0:
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and (op.addr & 0xFFFFFFFF) in DS:
                out.append("%X %s %s" % (ea, idc.get_func_name(f), idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\_disp1338.txt", "w").write("\n".join(out))
print(len(out))
