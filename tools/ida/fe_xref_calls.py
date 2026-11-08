# frontend agent: for each code ref to a data address, print the next call after it (fe_peek_args.txt: addr)
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_xref_calls.txt", "w")
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    ea = int(a, 16)
    seen = {}
    for x in idautils.XrefsTo(ea):
        p = x.frm
        for _ in range(6):
            p = idc.next_head(p)
            d = idc.GetDisasm(p)
            if d.startswith("call"):
                seen.setdefault(d, []).append(x.frm)
                break
    for d, l in sorted(seen.items(), key=lambda t: -len(t[1])):
        out.write("%5d %s  e.g. %08X\n" % (len(l), d, l[0]))
out.close()
print("ok")
