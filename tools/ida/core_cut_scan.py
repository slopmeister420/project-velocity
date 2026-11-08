import idc, idautils, ida_funcs, ida_ua
# GameBridge object (VsRender__5C8A0 = dword_6F1038): the next call after each `call 5C8A0` (mov ecx, eax; call X)
# -> research/core_cut_scan.txt "caller site -> callee"
out = []
for x in idautils.XrefsTo(0x5C8A0):
    if not idc.print_insn_mnem(x.frm).startswith("call"):
        continue
    h = x.frm
    tgt = None
    for _ in range(6):
        h = idc.next_head(h)
        if idc.print_insn_mnem(h) == "call":
            tgt = idc.get_operand_value(h, 0)
            break
    out.append("%08X %s -> %s" % (x.frm, idc.get_func_name(x.frm), idc.get_func_name(tgt) if tgt else "?"))
open(r"D:/projects/project poland rewrite/research/core_cut_scan.txt", "w").write("\n".join(out))
