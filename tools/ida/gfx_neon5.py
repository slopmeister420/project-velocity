# car shadow pass 0x11A6E0 inputs -> research/gfx/neon_dec5.c
import ida_hexrays, idc, ida_bytes, struct, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x71823C, 0x709D58, 0x706604, 0x6F5B7C]:
    v = ida_bytes.get_dword(ea)
    out.append("%X %08X" % (ea, v))
for ea in [0x71823C, 0x6F5B7C, 0x706604]:
    for x in idautils.XrefsTo(ea):
        out.append("xref %X <- %X %s : %s" % (ea, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
fs = [0x1638F0]
for x in idautils.XrefsTo(0x71823C):
    f = ida_funcs.get_func(x.frm)
    if f and f.start_ea not in fs and f.start_ea != 0x10D7E0:
        fs.append(f.start_ea)
for ea in fs:
    out.append("//==== %08X %s" % (ea, idc.get_func_name(ea)))
    try:
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\neon_dec5.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
