# RainSurface front-end reflection users -> research/gfx/ferefl.txt
import idc, idautils, ida_funcs
out = []
def callers(ea, depth, pre=""):
    for x in idautils.XrefsTo(ea):
        g = ida_funcs.get_func(x.frm)
        if not g:
            continue
        out.append("%s%X %s <- %X %s" % (pre, ea, idc.get_func_name(ea), x.frm, idc.get_func_name(g.start_ea)))
        if depth > 0:
            callers(g.start_ea, depth - 1, pre + "  ")
for ea in (0x1378E0, 0x136090, 0x136C50, 0x137B40, 0x137C50, 0x1367B0, 0x136840):
    callers(ea, 2)
open(r"D:\projects\project poland rewrite\research\gfx\ferefl.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
