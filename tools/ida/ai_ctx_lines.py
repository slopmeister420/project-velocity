import idc
# print 6 instructions before / 2 after each address in research/ai_dump/_req.txt -> _xref.txt (ai agent)
base = r"D:\projects\project poland rewrite\research\ai_dump"
addrs = [int(x, 16) for x in open(base + r"\_req.txt").read().split()]
out = open(base + r"\_xref.txt", "w")
for a in addrs:
    out.write("== %X %s\n" % (a, idc.get_func_name(a)))
    ea = a
    for _ in range(6):
        ea = idc.prev_head(ea)
    for _ in range(9):
        out.write("%X %s\n" % (ea, idc.GetDisasm(ea)))
        ea = idc.next_head(ea)
out.close()
