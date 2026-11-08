# physics agent: dwords at the addresses in research/damage_dump/f32.txt -> f32_out.txt (hex + f32)
import os, struct, ida_bytes

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
with open(os.path.join(ROOT, "f32_out.txt"), "w") as o:
    for a in open(os.path.join(ROOT, "f32.txt")).read().split():
        w = ida_bytes.get_dword(int(a, 16))
        o.write("%s %08X %r\n" % (a, w, struct.unpack("<f", struct.pack("<I", w))[0]))
print("ok")
