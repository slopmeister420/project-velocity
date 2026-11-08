# callees of the SetRenderState wrapper 0xC8DA0 and who else calls them (game code), with pushed args
import idautils, idc, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/cull_rs.txt"
with open(out, "w", encoding="utf-8") as f:
    fn = ida_funcs.get_func(0xC8DA0)
    callees = set()
    for h in idautils.Heads(fn.start_ea, fn.end_ea):
        f.write("%08X %s\n" % (h, idc.generate_disasm_line(h, 0)))
        if idc.print_insn_mnem(h) == "call":
            callees.add(idc.get_operand_value(h, 0))
    for c in callees:
        f.write("== callee %08X %s\n" % (c, idc.get_func_name(c)))
        for x in idautils.CodeRefsTo(c, 0):
            g = ida_funcs.get_func(x)
            args = []
            h = x
            for _ in range(6):
                h = idc.prev_head(h)
                if idc.print_insn_mnem(h) == "push":
                    args.append(idc.print_operand(h, 0))
            f.write("  %08X %s %s\n" % (x, idc.get_func_name(g.start_ea) if g else "?", args[:2]))
print("wrote", out)
