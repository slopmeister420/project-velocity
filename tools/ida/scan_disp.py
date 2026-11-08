import idautils, idc, ida_funcs, ida_ua
# instructions with a given memory displacement in game code
DISPS = [0x1E8]
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, ea):
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and (op.addr & 0xFFFFFFFF) in DISPS:
                out.append("%X\t%s\t%X\t%s" % (ea, ida_funcs.get_func_name(f), op.addr, idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\ai_dump\_disp_scan2.txt", "w").write("\n".join(out))
print(len(out))
