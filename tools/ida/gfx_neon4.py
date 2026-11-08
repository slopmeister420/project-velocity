# neon shader constants + setters -> research/gfx/neon_vals4.txt
import ida_hexrays, idc, ida_bytes, struct, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x686658, 0x68665C, 0x7184A8, 0x7184AC, 0x7184B0, 0x7184B4]:
    v = ida_bytes.get_dword(ea)
    out.append("%X %08X int %d float %r" % (ea, v, v if v < 0x80000000 else v - 0x100000000, struct.unpack("<f", struct.pack("<I", v))[0]))
for ea in [0x686658, 0x68665C]:
    for x in idautils.XrefsTo(ea):
        out.append("xref %X <- %X %s : %s" % (ea, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
for ea in [0x2A0DD0, 0x2A1760, 0x2A17D0, 0x2A1240]:
    out.append("//==== %08X %s" % (ea, idc.get_func_name(ea)))
    try:
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\neon_vals4.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
