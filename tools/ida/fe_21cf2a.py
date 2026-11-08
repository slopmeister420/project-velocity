import idc
out = []
ea = 0x21CF00
while ea < 0x21CF50:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0))); ea = idc.next_head(ea)
open(r"D:/projects/project poland rewrite/research/fe_21cf2a.txt", "w").write("\n".join(out))
