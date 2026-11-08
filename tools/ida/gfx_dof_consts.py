import idc, ida_bytes, struct
OUT = r"D:\projects\project poland rewrite\research\gfx\dof_consts.txt"
items = [("6868B8", 0x6868B8, 3), ("6868C4", 0x6868C4, 3), ("6868D0", 0x6868D0, 3), ("6868DC_i", 0x6868DC, 3),
         ("6868E8", 0x6868E8, 1), ("6868EC_i", 0x6868EC, 3), ("6868F8", 0x6868F8, 1), ("6868FC", 0x6868FC, 1),
         ("686900", 0x686900, 1), ("686904", 0x686904, 1), ("686908", 0x686908, 1), ("68690C", 0x68690C, 3)]
with open(OUT, "w") as f:
    for name, ea, n in items:
        for i in range(n):
            v = ida_bytes.get_dword(ea + 4 * i)
            f.write("%s[%d] %08X f=%r i=%d\n" % (name, i, v, struct.unpack("<f", struct.pack("<I", v))[0], v))
    for nm in ["aFff"]:
        ea = idc.get_name_ea_simple(nm)
        f.write("%s @ %X\n" % (nm, ea))
        for i in range(3):
            v = ida_bytes.get_dword(ea + 4 * i)
            f.write("  [%d] %08X f=%r\n" % (i, v, struct.unpack("<f", struct.pack("<I", v))[0]))
    for ea in [0x687550, 0x687551]:
        f.write("byte %X = %d\n" % (ea, ida_bytes.get_byte(ea)))
print("ok")
