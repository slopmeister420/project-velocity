# TrackFX 0x14B030 helpers part 2 -> research/gfx/hb_dec2.c, hb_vals2.txt
import ida_hexrays, idc, idautils, ida_funcs, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
def dec(ea):
    f = ida_funcs.get_func(ea)
    if not f:
        out.append("// no func %X" % ea)
        return
    try:
        s = str(ida_hexrays.decompile(f.start_ea))
    except Exception as ex:
        s = "// fail %s" % ex
    out.append("//==== %08X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    out.append(s)
for ea in [0x5C9C0, 0x5E070, 0x57F90, 0x1C68F0, 0x364BB0, 0x11B790, 0x42D40, 0x42DA0, 0x2B95D0, 0x24F90,
           0x19D350, 0x5B0F0, 0x5B2B0, 0x5A3B0]:
    dec(ea)
for tgt in [0x111B20]:
    out.append("// xrefs to %X:" % tgt)
    for x in idautils.CodeRefsTo(tgt, 0):
        out.append("//   %X in %s" % (x, idc.get_func_name(x)))
open(B + r"\hb_dec2.c", "w", encoding="utf-8").write("\n".join(out))
vals = []
for a in [0x6865E8, 0x6865F4, 0x6865F8, 0x686778, 0x68677C, 0x7184B0, 0x7065A4, 0x686780, 0x686784, 0x686788,
          0x68678C, 0x709ECC, 0x709ED8, 0x709EC8 + 16]:
    d = ida_bytes.get_dword(a)
    vals.append("%X %08X %r %s" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0], idc.get_name(a)))
for tgt in [0x709ED8, 0x709ECC, 0x7065A4, 0x7184B0]:
    vals.append("drefs to %X:" % tgt)
    for x in list(idautils.DataRefsTo(tgt))[:12]:
        vals.append("  %X in %s" % (x, idc.get_func_name(x)))
# GameManager call site of 0x14B030
for a in range(0xB9380, 0xB93D0):
    pass
vals.append(idc.generate_disasm_line(0xB93C1, 0))
ea = 0xB9370
while ea < 0xB93D0:
    vals.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea)
open(B + r"\hb_vals2.txt", "w", encoding="utf-8").write("\n".join(vals))
print("ok", len(out), len(vals))
