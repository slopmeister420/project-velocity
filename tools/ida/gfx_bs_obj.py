# writers of byte_6D6808 +8 (bonnet smoke count) -> research/gfx/bs_obj.txt
import idc, idautils, ida_funcs, ida_ua
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x6D6810]:
    out.append("== xrefs %X" % ea)
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s  %s" % (x.frm, f and idc.get_func_name(f.start_ea), idc.generate_disasm_line(x.frm, 0)))
# functions called with ecx = offset byte_6D6808
callees = set()
for x in idautils.XrefsTo(0x6D6808):
    nxt = idc.next_head(x.frm, x.frm + 16)
    if idc.print_insn_mnem(nxt) == "call":
        callees.add(idc.get_operand_value(nxt, 0))
# in those callees, stores to [reg+8]
for c in sorted(callees):
    f = ida_funcs.get_func(c)
    if not f:
        continue
    for ea in idautils.FuncItems(f.start_ea):
        d = idc.generate_disasm_line(ea, 0)
        if idc.print_insn_mnem(ea).startswith("mov") and "+8]" in d.split(",")[0]:
            out.append("store %X %s  %s" % (ea, idc.get_func_name(c), d))
    out.append("callee %X %s" % (c, idc.get_func_name(c)))
open(B + r"\bs_obj.txt", "w").write("\n".join(out))
print("ok")
