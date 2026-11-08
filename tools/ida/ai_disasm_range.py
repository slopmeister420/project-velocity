# disassemble [start,end) given as two hex numbers in research/ai_dump/_req.txt -> _xref.txt (ai agent)
import idc
a, b = [int(x, 16) for x in open(r"D:\projects\project poland rewrite\research\ai_dump\_req.txt").read().split()]
out = open(r"D:\projects\project poland rewrite\research\ai_dump\_xref.txt", "w")
ea = a
while ea < b:
    out.write("%X %s\n" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea, b + 16)
out.close()
