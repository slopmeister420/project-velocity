# physics agent stage 4b: misc queries -> research/phys_s4b_q.txt
import idc, idautils, ida_funcs
out = []
for n in ["AC::Renderer", "AC__Renderer"]:
    out.append("%s %X" % (n, idc.get_name_ea_simple(n)))
for a in [0x1DEBB0, 0x1E58C0, 0x1E3CB0, 0x1E32B0]:
    out.append("callers of %X:" % a)
    for x in idautils.CodeRefsTo(a, 0):
        g = ida_funcs.get_func(x)
        out.append("  %X %s" % (x, idc.get_func_name(x)))
    out.append("  type %s" % idc.get_type(a))
    f = ida_funcs.get_func(a)
    out.append("  ret: %s" % idc.generate_disasm_line(idc.prev_head(f.end_ea, a), 0))
open(r"D:\projects\project poland rewrite\research\phys_s4b_q.txt", "w").write("\n".join(out))
