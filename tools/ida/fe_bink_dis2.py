import idc, idautils
# disassemble the audio s16 conversion (fistp vs _ftol)
base = r"D:\projects\project poland rewrite\research"
dis = []
for f in (0x5429F0,):
    for h in idautils.FuncItems(f):
        dis.append("%X %s" % (h, idc.GetDisasm(h)))
open(base + r"\fe_bink_dis2.txt", "w").write("\n".join(dis))
