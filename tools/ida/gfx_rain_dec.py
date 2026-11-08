# camera rain 0x1341A0 / 0x135690: helpers, flags, constants -> research/gfx/rain_dec.c
import idc, ida_hexrays, idautils, idaapi, struct
out = []
def dec(ea):
    try:
        f = idaapi.get_func(ea)
        out.append("//==== %X %s" % (f.start_ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(f.start_ea)))
        out.append("// callers: " + " ".join("%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(f.start_ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
out.append("// functions 0x133900..0x136200")
for f in idautils.Functions(0x133900, 0x136200):
    out.append("//   %X %s" % (f, idc.get_func_name(f)))
for ea in (0x133CF0, 0x133D20, 0x134E90, 0x134EB0, 0x135630, 0x135D50, 0x135FB0, 0x136090, 0x5ABB0, 0x5AC40, 0x11DF60, 0x29F7F0):
    dec(ea)
for f in idautils.Functions(0x134DB0, 0x135690):
    if f not in (0x134E90, 0x134EB0, 0x135630):
        dec(f)
for name in ("byte_721DC8", "byte_721DC9", "byte_686F4C", "dword_7260E0", "dword_725E28"):
    ea = idc.get_name_ea_simple(name)
    out.append("// xrefs %s %X: %s" % (name, ea, " ".join("%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(ea))))
out.append("// GameManager call site")
ea = 0xB9F00
while ea < 0xB9F60:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0xB9F60)
out.append("// constants")
for a in range(0x686F10, 0x686FB0, 4):
    v = idc.get_wide_dword(a)
    out.append("%X = %08X %r" % (a, v, struct.unpack("<f", struct.pack("<I", v))[0]))
open(r"D:\projects\project poland rewrite\research\gfx\rain_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
