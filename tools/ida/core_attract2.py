import idc, idautils, ida_funcs, ida_bytes, struct
# callers of the attract mode methods, B3550 disassembly, .data constants
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_attract2.txt", "w")
for ea in (0xB3000, 0xB3060, 0xB30E0, 0xB3170, 0xB3430, 0xB3470, 0xB34A0, 0xB34D0, 0xB36B0, 0xB36F0):
    out.write("== callers %x\n" % ea)
    for x in idautils.XrefsTo(ea):
        out.write("  %x %s\n" % (x.frm, idc.get_func_name(x.frm)))
for a in (0x685698, 0x68569C, 0x6856A0):
    v = ida_bytes.get_dword(a)
    out.write("%x = %08x %r\n" % (a, v, struct.unpack("<f", struct.pack("<I", v))[0]))
out.write("== B3550 dis\n")
f = ida_funcs.get_func(0xB3550)
ea = f.start_ea
while ea < f.end_ea:
    out.write("%x  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, f.end_ea)
# aTracksLondon table
t = idc.get_name_ea_simple("aTracksLondon")
out.write("== table %x\n" % t)
for i in range(16):
    e = t + 68 * i
    a = idc.get_strlit_contents(e, -1, 0)
    b = idc.get_strlit_contents(e + 32, -1, 0)
    out.write("%d %r %r %d %d %d %d\n" % (i, a, b, ida_bytes.get_byte(e + 64), ida_bytes.get_byte(e + 65), ida_bytes.get_byte(e + 66), ida_bytes.get_byte(e + 67)))
out.close()
