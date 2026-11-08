# who starts the bonnet smoke systems: every caller of elParticleSystem start 0x2CF100, refs to byte_6D6808+156
import idc, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for x in idautils.CodeRefsTo(0x2CF100, 0):
    out.append("start %X in %s" % (x, idc.get_func_name(x)))
for a in (0x6D68A4, 0x6D6808):
    for x in idautils.DataRefsTo(a):
        out.append("ref %X <- %X %s | %s" % (a, x, idc.get_func_name(x), idc.GetDisasm(x)))
open(B + r"\bs_start.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
