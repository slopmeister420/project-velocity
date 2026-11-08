# danger particles (DangerZoneRender 0x1234A0..0x1238A0) helpers/data -> research/gfx/dp_dec.c
import ida_hexrays, idc, idautils, ida_funcs, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x1234A0, 0x1238A0, 0x1235F0, 0x123510, 0x719168, 0x719185, 0x719184, 0x719110, 0x719118]:
    out.append("== xrefs %X %s" % (ea, idc.get_name(ea)))
    for x in list(idautils.XrefsTo(ea))[:12]:
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s  %s" % (x.frm, f and idc.get_func_name(f.start_ea), idc.generate_disasm_line(x.frm, 0)))
for i in range(5):
    p = idc.get_wide_dword(0x6869A4 + 4 * i)
    out.append("name[%d] %X %s" % (i, p, idc.get_strlit_contents(p)))
for a in range(0x6869B8, 0x6869CC, 4):
    d = idc.get_wide_dword(a)
    out.append("%X %08X %r" % (a, d, struct.unpack('<f', struct.pack('<I', d))[0]))
for a in [0x452C0, 0x2D4A70, 0x2D4BD0, 0x2D5800, 0x2D59B0, 0x2D5920, 0x2D5960, 0x2D4EF0, 0x2ABDE0, 0x611B0,
          0x11DF60, 0x11DFF0, 0x5AC40, 0x5ABB0, 0x5ACD0, 0x14000, 0x13FC0]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
out.append("== static init near 0x4250FC")
ea = 0x425080
while ea < 0x425200:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x425300)
open(B + r"\dp_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
