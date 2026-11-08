# shader reset defaults (0x2A0AA0) + 0x2A9B00 strip draw + dword_687610 -> research/gfx/envmap5_dec.c
import ida_hexrays, idc, ida_funcs, idautils, ida_bytes
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x2A0AA0, 0x2A04C0]:
    try:
        out.append("// ===== %X\n%s" % (ea, str(ida_hexrays.decompile(ea))))
    except Exception as ex:
        out.append("// fail %X %s" % (ea, ex))
for x in idautils.XrefsTo(0x687610):
    out.append("// 687610 xref %X %s : %s" % (x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
out.append("// 687610 = %08X" % ida_bytes.get_dword(0x687610))
open(B + r"\envmap5_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
