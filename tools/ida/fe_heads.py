# frontend agent: print instructions from address for count (fe_peek_args.txt: "addr count")
import idc
a, n = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()[:2]
ea = int(a, 16)
for _ in range(int(n)):
    print("%08X %s" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea)
