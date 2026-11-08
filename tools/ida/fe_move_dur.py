import idautils, idc, ida_funcs, struct
out = []
for x in idautils.CodeRefsTo(0x272110, 0):
    f = ida_funcs.get_func(x)
    name = idc.get_func_name(x)
    ea = x
    prev = []
    for _ in range(3):
        ea = idc.prev_head(ea)
        prev.append(idc.GetDisasm(ea))
    out.append("%08X %s | %s" % (x, name, " ; ".join(reversed(prev))))
open(r"D:\projects\project poland rewrite\research\fe_move_dur.txt", "w").write("\n".join(out) + "\n")
