# car light flares static inits -> research/gfx/cf_dis.txt
import idc
out = []
ea = 0x425170
while ea < 0x425520:
    out.append("%06X %s" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea, 0x425530)
open(r"D:\projects\project poland rewrite\research\gfx\cf_dis.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
