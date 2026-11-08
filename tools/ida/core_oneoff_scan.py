import idc, idautils, ida_ua
# writes to [reg+3C8h] (VE+968, the one-off object read by VsAIRaceVehicle__23310) and reads of +0xA8 near
# 23310's callers -> research/core_oneoff_scan.txt
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        op = insn.ops[0]
        if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) == 0x3C8 and idc.print_insn_mnem(h) == "mov":
            out.append("W %08X %s | %s" % (h, idc.get_func_name(f), idc.GetDisasm(h)))
for x in idautils.XrefsTo(0x23310):
    out.append("X %08X %s" % (x.frm, idc.get_func_name(x.frm)))
open(r"D:/projects/project poland rewrite/research/core_oneoff_scan.txt", "w").write("\n".join(out))
