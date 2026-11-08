# decompile the render-state switch 0xC8190; list call targets with their xref counts
import idautils, idc, ida_funcs, ida_hexrays
out = "D:/projects/project poland rewrite/research/gfx/cull_c8190.txt"
with open(out, "w", encoding="utf-8") as f:
    f.write(str(ida_hexrays.decompile(0xC8190)))
    fn = ida_funcs.get_func(0xC8190)
    seen = set()
    for h in idautils.Heads(fn.start_ea, fn.end_ea):
        if idc.print_insn_mnem(h) == "call":
            t = idc.get_operand_value(h, 0)
            if t in seen:
                continue
            seen.add(t)
            refs = [x for x in idautils.CodeRefsTo(t, 0)]
            outside = ["%08X" % x for x in refs if not (fn.start_ea <= x < fn.end_ea)]
            f.write("\n// call %08X %s refs %d outside %s\n" % (t, idc.get_func_name(t), len(refs), outside[:12]))
print("wrote", out)
