import idc, idautils, ida_ua
# functions using memory displacements listed in research/core_dispscan.in (hex per line) -> research/core_dispscan.txt
want = set(int(x, 16) for x in open(r"D:/projects/project poland rewrite/research/core_dispscan.in").read().split())
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        for op in insn.ops:
            if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) in want:
                out.append("%08X %s | %s" % (h, idc.get_func_name(h), idc.GetDisasm(h)))
open(r"D:/projects/project poland rewrite/research/core_dispscan.txt", "w").write("\n".join(out))
