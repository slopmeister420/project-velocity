# override shader use in the renderer -> research/gfx/cs_dec4.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
def dec(a):
    f = ida_funcs.get_func(a)
    if not f or f.start_ea in seen:
        return
    seen.add(f.start_ea)
    out.append("//==== %X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
dec(0x2A05E0)
for x in idautils.XrefsTo(0x2A05E0):
    out.append("// 2A05E0 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
for x in idautils.XrefsTo(0x2A4960):
    out.append("// 2A4960 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
    dec(x.frm)
open(B + r"\cs_dec4.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
