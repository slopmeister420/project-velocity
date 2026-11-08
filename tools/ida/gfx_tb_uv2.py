# find how eMS code 22 (uv anim) is applied -> research/gfx/tb_uv2.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x2A4960, 0x3028A0, 0x825C0]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
for vt in [0x66B34C]:
    out.append("== xrefs vt %X" % vt)
    for x in idautils.XrefsTo(vt):
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s" % (x.frm, f and idc.get_func_name(f.start_ea)))
for f in [0x3015F0]:
    out.append("== xrefs %X" % f)
    for x in idautils.XrefsTo(f):
        g = ida_funcs.get_func(x.frm)
        out.append("  %X %s" % (x.frm, g and idc.get_func_name(g.start_ea)))
open(B + r"\tb_uv2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
