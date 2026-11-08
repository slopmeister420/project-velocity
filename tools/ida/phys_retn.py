# physics agent: retn sizes and current prototypes of a few functions -> research/_retn.txt
import idc, ida_funcs, idautils
R = r"D:\projects\project poland rewrite\research"
out = []
for a in [0x19FD80, 0x1E18E0, 0x1E1A20, 0x1B3500, 0x1B3850, 0x19D310, 0x19D350, 0x19D3E0, 0x762F0, 0x19C390]:
    f = ida_funcs.get_func(a)
    rets = set()
    for ea in idautils.FuncItems(a):
        m = idc.print_insn_mnem(ea)
        if m.startswith("ret"):
            rets.add(idc.generate_disasm_line(ea, 0))
    out.append("%X  %s  | %s" % (a, sorted(rets), idc.get_type(a)))
open(R + r"\_retn.txt", "w").write("\n".join(out) + "\n")
