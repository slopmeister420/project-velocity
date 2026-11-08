# physics agent: jump cam / fury globals (dword + f32) -> research/phys_dump/jumpcam_consts.txt
import struct, ida_bytes

OUT = r"D:\projects\project poland rewrite\research\phys_dump\jumpcam_consts.txt"
with open(OUT, "w") as o:
    for a in (0x6889F4, 0x6889FC, 0x688A00, 0x688A0C, 0x75B278, 0x749110, 0x737AB0):
        w = ida_bytes.get_dword(a)
        o.write("%06X %08X %r\n" % (a, w, struct.unpack("<f", struct.pack("<I", w))[0]))
print("ok")
