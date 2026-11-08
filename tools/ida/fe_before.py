# frontend agent: print the N instructions before and 2 after each address (fe_peek_args.txt: n addr...)
import idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
n = int(a[0])
for s in a[1:]:
    ea = int(s, 16)
    p = ea
    lines = []
    for _ in range(n):
        p = idc.prev_head(p)
        lines.append("  %08X  %s" % (p, idc.GetDisasm(p)))
    print("==", s)
    print("\n".join(reversed(lines)))
    q = ea
    for _ in range(3):
        print("  %08X  %s" % (q, idc.GetDisasm(q)))
        q = idc.next_head(q)
