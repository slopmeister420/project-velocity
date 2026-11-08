# TrackFX headlight beams 0x14B030: helpers, callers, constants -> research/gfx/hb_dec.c, hb_vals.txt
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
for ea in [0x5A040, 0x57200, 0x5C820, 0x23DD0, 0x2ABDE0, 0x11DE40, 0x111B20, 0x127340, 0x761B0, 0x5FFB0,
           0xC8CF0, 0x5ACD0, 0x5ABB0, 0x59140, 0x2B11C0]:
    dec(ea)
for tgt in [0x14B030, 0x14BE40, 0x14C430, 0x149020, 0x5A040]:
    out.append("// xrefs to %X:" % tgt)
    for x in idautils.CodeRefsTo(tgt, 0):
        out.append("//   %X in %s" % (x, idc.get_func_name(x)))
open(B + r"\hb_dec.c", "w", encoding="utf-8").write("\n".join(out))
vals = []
for a in list(range(0x687280, 0x6873C0, 4)) + [0x737498, 0x737490, 0x7374AC, 0x73749C, 0x6865F0]:
    d = ida_bytes.get_dword(a)
    vals.append("%X %08X %r %s" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0], idc.get_name(a)))
for tgt in [0x737498, 0x737494]:
    vals.append("drefs to %X:" % tgt)
    for x in idautils.DataRefsTo(tgt):
        vals.append("  %X in %s" % (x, idc.get_func_name(x)))
open(B + r"\hb_vals.txt", "w", encoding="utf-8").write("\n".join(vals))
print("ok", len(out), len(vals))
