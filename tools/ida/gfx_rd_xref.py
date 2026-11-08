# roaddetail list dword_6D5328: xrefs + DrawList functions -> research/gfx/rd_xref.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
def dec(a):
    f = ida_funcs.get_func(a)
    if not f or f.start_ea in seen:
        return
    seen.add(f.start_ea)
    out.append("//==== %08X %s  [%s]" % (f.start_ea, idc.get_func_name(f.start_ea), idc.get_type(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
for g in (0x6D5328, 0x6D531C, 0x6D5324, 0x6D5320, 0x6D532C, 0x6D5338, 0x6D5330, 0x6D5334):
    for x in idautils.DataRefsTo(g):
        out.append("// ref %X -> %X in %s" % (g, x, idc.get_func_name(x)))
for x in idautils.DataRefsTo(0x6D5328):
    dec(x)
dec(0x2C21A0)
for x in idautils.CodeRefsTo(0x2C21A0, 0):
    out.append("// DrawList__2C21A0 caller %X in %s" % (x, idc.get_func_name(x)))
# functions in the DrawList region
f = ida_funcs.get_func(0x2C21A0)
a = f.start_ea - 0x600
while a < f.end_ea + 0xA00:
    g = ida_funcs.get_func(a)
    if g:
        out.append("// fn %X %s" % (g.start_ea, idc.get_func_name(g.start_ea)))
        a = g.end_ea
    else:
        a += 1
open(B + r"\rd_xref.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
