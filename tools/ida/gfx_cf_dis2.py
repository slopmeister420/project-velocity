# car light flares static inits part 2 -> research/gfx/cf_dis2.txt
import idc, struct
out = []
ea = 0x425500
while ea < 0x425600:
    out.append("%06X %s" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea, 0x425610)
for a in (0x63BE9C, 0x6249C0, 0x6275CC, 0x625310, 0x6275EC, 0x625750, 0x63BE98, 0x625178):
    d = idc.get_wide_dword(a)
    out.append("%X %08X %r" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
open(r"D:\projects\project poland rewrite\research\gfx\cf_dis2.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
