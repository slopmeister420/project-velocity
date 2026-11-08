# disassemble 0x1270E0 and dump the flash constants
import idc, idautils, ida_bytes, struct
out = "D:/projects/project poland rewrite/research/gfx/s9_dis.txt"
with open(out, "w", encoding="utf-8") as f:
    for h in idautils.FuncItems(0x1270E0):
        f.write("%08X %s\n" % (h, idc.GetDisasm(h)))
    for a in range(0x686B10, 0x686B50, 4):
        d = ida_bytes.get_dword(a)
        f.write("%08X %s %08X %r\n" % (a, idc.get_name(a), d, struct.unpack("<f", struct.pack("<I", d))[0]))
print("ok")
