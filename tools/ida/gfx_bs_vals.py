# values + refs of the bonnet smoke scalars -> research/gfx/bs_vals.txt
import idc, idautils, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in (0x6852DC, 0x6D67A8, 0x6D67AC, 0x6D67B0, 0x6D67B4, 0x6D67B8, 0x6D67F0):
    d = ida_bytes.get_dword(a)
    out.append("%X = %08X %r" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
    for x in idautils.DataRefsTo(a):
        out.append("   <- %X %s | %s" % (x, idc.get_func_name(x), idc.GetDisasm(x)))
open(B + r"\bs_vals.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
