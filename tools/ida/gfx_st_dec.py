import idc, struct
OUT = r"D:\projects\project poland rewrite\research\gfx\st_puddle_consts.txt"
with open(OUT, "w") as f:
    for a in range(0x686EF0, 0x686F14, 4):
        d = idc.get_wide_dword(a)
        f.write("%X %08X %r\n" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
    d = idc.get_wide_dword(0x686EA8)
    f.write("686EA8 %08X\n" % d)
print("ok")
