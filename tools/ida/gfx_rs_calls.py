# calls to the SetRenderState wrapper StringManager__C8DA0 (0xC8DA0): print the two pushed args per call site
import idautils, ida_funcs, idc, ida_ua
out = "D:/projects/project poland rewrite/research/gfx/rs_calls.txt"
target = 0xC8DA0
with open(out, "w", encoding="utf-8") as f:
    for x in idautils.CodeRefsTo(target, 0):
        args = []
        h = x
        for _ in range(8):
            h = idc.prev_head(h)
            if idc.print_insn_mnem(h) == "push":
                args.append(idc.print_operand(h, 0))
                if len(args) == 2:
                    break
        fn = ida_funcs.get_func(x)
        f.write("%08X %s state=%s value=%s\n" % (x, idc.get_func_name(fn.start_ea) if fn else "?", args[1] if len(args) > 1 else "?", args[0] if args else "?"))
print("wrote", out)
