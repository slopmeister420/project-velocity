import idc
# disassemble research/core_dis_in.txt "start end" (hex) to research/core_dis.txt
base = r"D:\projects\project poland rewrite\research"
s, e = [int(x, 16) for x in open(base + r"\core_dis_in.txt").read().split()]
out = open(base + r"\core_dis.txt", "w")
ea = s
while ea < e and ea != idc.BADADDR:
    out.write("%X  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, e)
out.close()
