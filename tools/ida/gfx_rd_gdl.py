# GDL_RelocateModel and helpers (shader +4 flags writers) -> research/gfx/rd_gdl.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
def dec(a):
    f = ida_funcs.get_func(a)
    if not f or f.start_ea in seen:
        return None
    seen.add(f.start_ea)
    out.append("//==== %08X %s  [%s]" % (f.start_ea, idc.get_func_name(f.start_ea), idc.get_type(f.start_ea)))
    try:
        s = str(ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        s = "// fail %s" % ex
    out.append(s)
    return f
f = dec(0x9EF30)
# direct callees of 0x9EF30 inside game code
if f:
    for h in idautils.FuncItems(f.start_ea):
        for x in idautils.CodeRefsFrom(h, 0):
            g = ida_funcs.get_func(x)
            if g and g.start_ea != f.start_ea and 0x11000 <= g.start_ea < 0xC0000:
                dec(g.start_ea)
for x in idautils.CodeRefsTo(0x9EF30, 0):
    out.append("// caller %X in %s" % (x, idc.get_func_name(x)))
open(B + r"\rd_gdl.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
