# car light flares (DangerZoneRender 0x123E60 family, object unk_719450) -> research/gfx/cf_dec.c
import idc, ida_hexrays, idautils, ida_bytes, struct
out = []
def dec(ea):
    try:
        out.append("//==== %X %s" % (ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
for ea in (0x1271B0, 0x127340, 0x127360, 0x127390, 0x11DFF0, 0x11DF60, 0x5ABB0, 0x5ACD0, 0x5C820, 0x11BA50, 0x42DD0, 0x42E00, 0x42DA0, 0x57F90, 0x7CF40, 0x12BC40):
    dec(ea)
def xr(ea, label):
    out.append("// xrefs %s %X: %s" % (label, ea, " ".join("%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(ea))))
for ea in (0x719450, 0x123A20, 0x123930, 0x123970, 0x123E60, 0x127080, 0x1270E0, 0x719440, 0x719394, 0x719444, 0x719448, 0x719398, 0x7192B8, 0x7193A0, 0x719358, 0x7191A8, 0x719188, 0x7192AC, 0x686A10, 0x686B28):
    xr(ea, idc.get_name(ea))
out.append("// constants 0x6869C0..0x686B50")
for ea in range(0x6869C0, 0x686B50, 4):
    d = idc.get_wide_dword(ea)
    f = struct.unpack("<f", struct.pack("<I", d))[0]
    s = ""
    if 0x600000 < d < 0x800000:
        s = idc.get_strlit_contents(d) or b""
        s = s.decode("latin1", "replace")
    out.append("%X %s %08X %r %s" % (ea, idc.get_name(ea), d, f, s))
for ea in (0x63BE28, 0x63BE24):
    out.append("%X str %r" % (ea, idc.get_strlit_contents(ea)))
open(r"D:\projects\project poland rewrite\research\gfx\cf_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
