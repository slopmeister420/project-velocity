import idc, ida_bytes, struct
OUT = r"D:\projects\project poland rewrite\research\gfx\fse_tbl.txt"
def f32(ea):
    return struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(ea)))[0]
with open(OUT, "w") as o:
    for ea in range(0x687580, 0x687650, 4):
        d = ida_bytes.get_dword(ea)
        s = idc.get_strlit_contents(d) if 0x600000 <= d < 0x700000 else None
        o.write("%x %08x %r %s\n" % (ea, d, f32(ea), s))
    # globals used by fse / dangerzone / bloom / motion blur
    for ea in [0x6868AC,0x6868B0,0x6868B4,0x6868B8,0x6868BC,0x6868C0,0x6868C4,0x6868C8,0x6868CC,0x6868D0,0x6868D4,
               0x6868D8,0x6868DC,0x6868E0,0x6868E4,0x6868E8,0x6868EC,0x6868F0,0x6868F4,0x6868F8,0x6868FC,0x686900,
               0x686904,0x686908,0x68690C,0x686910,0x686914,0x7190F4,0x7190F0,0x718AB8,0x6875D0,0x6875D4]:
        o.write("G %x %08x %r\n" % (ea, ida_bytes.get_dword(ea), f32(ea)))
print("ok")
