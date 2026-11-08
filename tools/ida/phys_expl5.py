# physics agent: explosion port - callers of the world reset 0x19C390 and of the AI friction spot caller 0x19DD0
import idautils, ida_funcs, ida_name
R = r"D:\projects\project poland rewrite\research"
out = []
for a in [0x19C390, 0x19DD0, 0x1DE400]:
    out.append("== callers of %X %s" % (a, ida_name.get_name(a)))
    for x in idautils.CodeRefsTo(a, 0):
        f = ida_funcs.get_func(x)
        out.append("  %X in %X %s" % (x, f.start_ea if f else 0, ida_name.get_name(f.start_ea) if f else "?"))
    for x in idautils.DataRefsTo(a):
        out.append("  data %X" % x)
open(R + r"\phys_expl5.txt", "w").write("\n".join(out) + "\n")
