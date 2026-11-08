# every game-code "push 93h" (D3DRS_CULLMODE) with the previous push (value) and the following call
import idautils, idc, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/cull_push93.txt"
with open(out, "w", encoding="utf-8") as f:
    for ea in idautils.Heads(0x11000, 0x3DDD20):
        if idc.print_insn_mnem(ea) != "push" or idc.print_operand(ea, 0) != "93h":
            continue
        p = idc.prev_head(ea)
        n = idc.next_head(ea)
        fn = ida_funcs.get_func(ea)
        f.write("%08X %s | %s | %s\n" % (ea, idc.get_func_name(fn.start_ea) if fn else "?", idc.generate_disasm_line(p, 0), idc.generate_disasm_line(n, 0)))
    # direct calls of the CullMode setter from game code
    for x in idautils.CodeRefsTo(0x440E80, 0):
        fn = ida_funcs.get_func(x)
        f.write("ref440E80 %08X %s | %s\n" % (x, idc.get_func_name(fn.start_ea) if fn else "?", idc.generate_disasm_line(idc.prev_head(x), 0)))
print("wrote", out)
