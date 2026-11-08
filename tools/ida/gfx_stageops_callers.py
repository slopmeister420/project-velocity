# callers of CShaderManager::SetStageOps 0x2A7120: the 6 pushed args before each call -> research/gfx/stageops_callers.txt
import idautils, idc, ida_funcs
out = []
for x in idautils.CodeRefsTo(0x2A7120, 0):
    f = ida_funcs.get_func(x)
    fn = idc.get_func_name(x)
    ea = x
    lines = []
    for _ in range(9):
        ea = idc.prev_head(ea)
        lines.append(idc.generate_disasm_line(ea, 0))
    out.append("%X %s: %s" % (x, fn, " | ".join(reversed(lines))))
open(r"D:/projects/project poland rewrite/research/gfx/stageops_callers.txt", "w").write("\n".join(out) + "\n")
