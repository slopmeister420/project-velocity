# camera rain: sprite projection / draw, ray constants -> research/gfx/rain_dec3.c
import idc, ida_hexrays, idautils, idaapi, struct
out = []
def dec(ea):
    try:
        f = idaapi.get_func(ea)
        out.append("//==== %X %s" % (f.start_ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
for ea in (0x2A7840, 0x2A7F90):
    dec(ea)
for a in (0x63C91C, 0x63C920, 0x7805B0):
    v = idc.get_wide_dword(a)
    out.append("// %X = %08X %r" % (a, v, struct.unpack("<f", struct.pack("<I", v))[0]))
out.append("// xrefs 7805B0: " + " ".join("%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(0x7805B0)))
open(r"D:\projects\project poland rewrite\research\gfx\rain_dec3.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
