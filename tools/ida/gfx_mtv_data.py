# dump the MTV glitch overlay data (LightStreak.cpp 0x1300D0..): names, scales, sequences, tweak floats
import ida_bytes, idc, struct
out = "D:/projects/project poland rewrite/research/gfx/mtv_data.txt"
def f32(ea):
    return struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(ea)))[0]
with open(out, "w", encoding="utf-8") as f:
    f.write("names off_686C40:\n")
    for i in range(14):
        p = ida_bytes.get_dword(0x686C40 + 4 * i)
        f.write("  %d %08X %s\n" % (i, p, idc.get_strlit_contents(p, -1, 0)))
    f.write("scales dword_686C78:\n")
    for i in range(14):
        ea = 0x686C78 + 4 * i
        f.write("  %d %08X %08X %r\n" % (i, ea, ida_bytes.get_dword(ea), f32(ea)))
    f.write("floats 686E70..686EA8:\n")
    for ea in range(0x686E70, 0x686EA8, 4):
        f.write("  %08X %s %08X %r\n" % (ea, idc.get_name(ea), ida_bytes.get_dword(ea), f32(ea)))
    for ea in (0x625378, 0x636D10, 0x624968, 0x6249C0, 0x624E14):
        f.write("  %08X %s %08X %r\n" % (ea, idc.get_name(ea), ida_bytes.get_dword(ea), f32(ea)))
    f.write("CreateShader name 624E14: %r\n" % idc.get_strlit_contents(0x624E14, -1, 0))
    f.write("sequences 686E50:\n")
    for k in range(4):
        e = 0x686E50 + 8 * k
        p = ida_bytes.get_dword(e)
        n = ida_bytes.get_word(e + 4)
        q = ida_bytes.get_word(e + 6)
        f.write("  seq %d frames %08X count %d per %d\n" % (k, p, n, q))
        for j in range(n * q):
            a = p + 16 * j
            f.write("    %d: x %r y %r size %r tex %d\n" % (j, f32(a), f32(a + 4), f32(a + 8), struct.unpack("<i", struct.pack("<I", ida_bytes.get_dword(a + 12)))[0]))
print("wrote", out)
