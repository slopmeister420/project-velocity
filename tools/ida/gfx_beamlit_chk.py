# beam _Lit lights: who calls SetLight (via Renderer 0x2A8BE0) and in which frame order -> research/gfx/beamlit_chk.c
import idc, ida_hexrays, idautils, ida_funcs
out = []
def dec(ea):
    try:
        out.append("//==== %X %s" % (ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
def callers(ea, depth, pre=""):
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        if not f:
            continue
        out.append("%s%X %s <- %X %s" % (pre, ea, idc.get_func_name(ea), x.frm, idc.get_func_name(f.start_ea)))
        if depth > 0:
            callers(f.start_ea, depth - 1, pre + "  ")
dec(0x2A8BE0)
dec(0x2A8FD0)
callers(0x2A8BE0, 3)
open(r"D:\projects\project poland rewrite\research\gfx\beamlit_chk.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
