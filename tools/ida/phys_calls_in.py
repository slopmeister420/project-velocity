# physics agent: list call sites (and targets) inside the functions given in ADDRS
import idautils, idc, ida_funcs
ADDRS = [0x19D3E0, 0x1A7060]
WANT = [0x1B3F20, 0x1B3F40, 0x1B4640, 0x1B4650]
out = []
for fa in ADDRS:
    f = ida_funcs.get_func(fa)
    for ea in idautils.FuncItems(f.start_ea):
        if idc.print_insn_mnem(ea) == "call":
            t = idc.get_operand_value(ea, 0)
            if t in WANT:
                out.append("%X in %X -> %X %s" % (ea, fa, t, idc.get_func_name(t)))
for a in WANT:
    out.append("%X %s: %s" % (a, idc.get_func_name(a), " | ".join(idc.GetDisasm(e) for e in list(idautils.FuncItems(a))[:12])))
open(r"D:\projects\project poland rewrite\research\phys_calls_in.txt", "w").write("\n".join(out) + "\n")
