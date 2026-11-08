import idc, idautils, ida_ua
# VE camera fields (+4392, +4436..+4444, +5008..+5020): every displacement use -> research/phys_camscan.txt
want = set([0x1128, 0x1154, 0x1158, 0x115C, 0x1390, 0x1394, 0x1398, 0x139C])
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        for op in insn.ops:
            if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) in want:
                out.append("%08X %s | %s" % (h, idc.get_func_name(h), idc.GetDisasm(h)))
out.append("== callers of 1D1610")
for x in idautils.CodeRefsTo(0x1D1610, 0):
    out.append("%08X %s" % (x, idc.get_func_name(x)))
open(r"D:/projects/project poland rewrite/research/phys_camscan.txt", "w").write("\n".join(out))
