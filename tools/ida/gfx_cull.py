# find game-code pushes of the xbox D3DCULL_CW / CCW values (0x900 / 0x901) and the call that follows
import idautils, ida_funcs, idc
out = "D:/projects/project poland rewrite/research/gfx/cull_calls.txt"
with open(out, "w", encoding="utf-8") as f:
    for ea in idautils.Heads(0x11000, 0x3DDD20):
        if idc.print_insn_mnem(ea) not in ("push", "mov"):
            continue
        ops = idc.print_operand(ea, 0) + "," + idc.print_operand(ea, 1)
        if "900h" not in ops and "901h" not in ops:
            continue
        h = ea
        call = "?"
        for _ in range(6):
            h = idc.next_head(h)
            if idc.print_insn_mnem(h) == "call":
                call = idc.print_operand(h, 0)
                break
        fn = ida_funcs.get_func(ea)
        f.write("%08X %s %s | %s\n" % (ea, idc.get_func_name(fn.start_ea) if fn else "?", idc.generate_disasm_line(ea, 0), call))
print("wrote", out)
