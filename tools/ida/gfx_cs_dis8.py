import idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
ea = 0x156C00
while ea < 0x1570C3:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea)
open(B + r"\cs_dis8.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
