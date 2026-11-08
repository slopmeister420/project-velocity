# camera rain: init / shader / sprite helpers -> research/gfx/rain_dec2.c
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
for ea in (0x133AB0, 0x133CD0, 0x134120, 0x135C20, 0x135C70, 0x135CD0, 0x135D00, 0x135DC0, 0x135E20, 0x135EF0, 0x135F70,
           0x2A7AE0, 0x2AA770, 0x2AAA70, 0x24CC0, 0x79170, 0x790F0, 0x2D0B70, 0x2D11D0, 0x79080):
    dec(ea)
out.append("// 625678 = %08X %r" % (idc.get_wide_dword(0x625678), struct.unpack("<f", struct.pack("<I", idc.get_wide_dword(0x625678)))[0]))
# disassembly of 0x136090 case 4 tail and the 134E90/134EB0 bodies (fp args)
for s, e in ((0x134E90, 0x134EF0), (0x1365A0, 0x136720)):
    out.append("// disasm %X..%X" % (s, e))
    ea = s
    while ea < e:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea, e)
open(r"D:\projects\project poland rewrite\research\gfx\rain_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
