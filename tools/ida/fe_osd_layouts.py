# frontend agent: COSDInGameScreen::Create layout tables (27 x (x, y) floats each) -> research/fe_osd_layouts.txt
import idc, struct
out = open(r"D:\projects\project poland rewrite\research\fe_osd_layouts.txt", "w")
for base in (0x6860F0, 0x6861C8, 0x6862A0, 0x686378, 0x686450):
    out.write("== %08X\n" % base)
    for i in range(27):
        x = idc.get_wide_dword(base + 8 * i)
        y = idc.get_wide_dword(base + 8 * i + 4)
        fx = struct.unpack("<f", struct.pack("<I", x))[0]
        fy = struct.unpack("<f", struct.pack("<I", y))[0]
        out.write("%2d %08X %08X %r %r\n" % (i, x, y, fx, fy))
out.close()
print("ok")
