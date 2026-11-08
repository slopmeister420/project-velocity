# RainSurface reflection helpers + constants -> research/gfx/rd_misc.c
import ida_hexrays, idc, idautils, ida_funcs, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x15C920, 0x15CA70, 0x15C770, 0x15BBF0, 0x2EEFE0, 0x2EF030, 0x2EEE20, 0x2EF160, 0x15E990, 0x2A0730,
          0x11F260, 0x137610, 0x131F50, 0x135120]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
def f32(a):
    return struct.unpack("<f", ida_bytes.get_bytes(a, 4))[0]
for a in [0x687658, 0x68765C, 0x687660, 0x687664, 0x686FC4, 0x686FC8, 0x686FCC, 0x686FD0, 0x686FD4, 0x686FD8,
          0x7263DC, 0x7266D8, 0x748980, 0x748984, 0x748988, 0x686FB4, 0x686FB8, 0x736940]:
    d = ida_bytes.get_dword(a)
    out.append("// %X dword %08X f32 %r byte %d  name %s" % (a, d, f32(a), ida_bytes.get_byte(a), idc.get_name(a)))
for g in (0x687658, 0x68765C, 0x687660, 0x687664, 0x7263DC, 0x7266D8, 0x748980, 0x748984, 0x748988, 0x686FCC, 0x686FC8, 0x686FD0):
    for x in idautils.DataRefsTo(g):
        out.append("// ref %X <- %X %s | %s" % (g, x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
open(B + r"\rd_misc.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
