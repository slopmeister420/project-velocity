# helpers / callers / constants of TrailBlazerRender -> research/gfx/tb_dec2.c
import ida_hexrays, idc, idautils, ida_funcs, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x14DF70, 0x14E2A0, 0x14E530, 0x14E710, 0x14DEF0, 0x7374D0]:
    out.append("== xrefs %X %s" % (ea, idc.get_name(ea)))
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s" % (x.frm, f and idc.get_func_name(f.start_ea)))
for a in [0x14DEF0, 0x28460, 0xBF670, 0xBFA50, 0x12A680, 0x11DDF0, 0x301520, 0x3016E0, 0x2A2380, 0xC6240, 0x613E0]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
out.append("== constants")
for a in range(0x6873D0, 0x687430, 4):
    d = idc.get_wide_dword(a)
    out.append("%X %08X %r %d" % (a, d, struct.unpack('<f', struct.pack('<I', d))[0], d))
for a in [0x7374C0, 0x7187E0]:
    out.append("%X %08X" % (a, idc.get_wide_dword(a)))
open(B + r"\tb_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
