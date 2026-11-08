# beam _Lit lights part 2: DrawList args in GameManager 0xB9010, the hud caller chain -> research/gfx/beamlit_chk2.txt
import idc, idautils, ida_funcs
out = []
f = ida_funcs.get_func(0xB9010)
for x in idautils.XrefsTo(0x2C2A00):
    if f.start_ea <= x.frm < f.end_ea:
        out.append("== call %X" % x.frm)
        ea = x.frm
        lines = []
        for _ in range(8):
            ea = idc.prev_head(ea)
            lines.append("  %06X %s" % (ea, idc.GetDisasm(ea)))
        out.extend(reversed(lines))
def callers(ea, depth, pre=""):
    for x in idautils.XrefsTo(ea):
        g = ida_funcs.get_func(x.frm)
        if not g:
            continue
        out.append("%s%X %s <- %X %s" % (pre, ea, idc.get_func_name(ea), x.frm, idc.get_func_name(g.start_ea)))
        if depth > 0:
            callers(g.start_ea, depth - 1, pre + "  ")
callers(0xEA2B0, 3)
callers(0xB9010, 1)
for ea in (0xB975D, 0xB9760):
    out.append("%06X %s" % (ea, idc.GetDisasm(ea)))
open(r"D:\projects\project poland rewrite\research\gfx\beamlit_chk2.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
