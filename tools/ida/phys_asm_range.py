# physics agent: disassemble [START, END) to research/_asm.txt (range from research/_asm_range.txt)
import idc
R = r"D:\projects\project poland rewrite\research"
s, e = [int(x, 16) for x in open(R + r"\_asm_range.txt").read().split()]
out = []
ea = s
while ea < e:
    out.append("%08X  %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, e + 16)
open(R + r"\_asm.txt", "w").write("\n".join(out))
