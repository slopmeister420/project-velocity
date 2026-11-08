# print f32 / u32 at the addresses in research/gfx/vals_in.txt -> research/gfx/vals_out.txt
import ida_bytes, struct
base = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(base + r"\vals_in.txt"):
    l = l.strip()
    if not l: continue
    a = int(l, 16)
    u = ida_bytes.get_dword(a)
    out.append("%X u32 %08X f32 %r" % (a, u, struct.unpack("<f", struct.pack("<I", u))[0]))
open(base + r"\vals_out.txt", "w").write("\n".join(out) + "\n")
