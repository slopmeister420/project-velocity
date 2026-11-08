# callers of CarRender 0x115920 (neon set) + 0x11A6E0 + string 63B730 -> research/gfx/neon_dec2.c
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
p = ida_bytes.get_dword(0x63B730)
out.append("// off_63B730 -> %X %r" % (p, idc.get_strlit_contents(p)))
seen = set()
for x in idautils.XrefsTo(0x115920):
    f = ida_funcs.get_func(x.frm)
    out.append("// caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
    if f and f.start_ea not in seen:
        seen.add(f.start_ea)
for ea in list(seen) + [0x11A6E0, 0x1100B0]:
    out.append("//==== %08X %s" % (ea, idc.get_func_name(ea)))
    try:
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
for x in idautils.XrefsTo(0x1100B0):
    out.append("// 1100B0 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
for x in idautils.XrefsTo(0x11A6E0):
    out.append("// 11A6E0 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
open(B + r"\neon_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
