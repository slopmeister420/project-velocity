# static inits of the shadow colours + the byte_6F5B7C tweak -> research/gfx/neon_dis6.txt
import idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for (s, e) in [(0x424A70, 0x424B10), (0xB4780, 0xB47E0)]:
    out.append("==== %X" % s)
    ea = s
    while ea < e:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
open(B + r"\neon_dis6.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
