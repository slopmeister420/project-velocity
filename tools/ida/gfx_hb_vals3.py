# a few floats for the TrackFX beams -> research/gfx/hb_vals3.txt
import ida_bytes, struct, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x685094, 0x629238, 0x6249C0, 0x63DF88]:
    d = ida_bytes.get_dword(a)
    out.append("%X %08X %r" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
out.append("63DF88 str: %r" % idc.get_strlit_contents(ida_bytes.get_dword(0x63DF88) if ida_bytes.get_dword(0x63DF88) > 0x10000 else 0x63DF88))
out.append("63DF88 raw: %r" % idc.get_strlit_contents(0x63DF88))
open(B + r"\hb_vals3.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
