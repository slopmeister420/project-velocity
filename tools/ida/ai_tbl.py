import idc, struct
out = []
for i in range(8):
    p = idc.get_wide_dword(0x625168 + 4 * i)
    f = struct.unpack("<f", struct.pack("<I", idc.get_wide_dword(p)))[0] if p > 0x10000 else None
    out.append("%d %X -> %s name=%s" % (i, p, f, idc.get_name(0x625168 + 4 * i)))
open(r"D:\projects\project poland rewrite\research\ai_tbl.txt", "w").write("\n".join(out))
