# physics agent (handling review): code xrefs to the addresses in research/phys_hr_addrs.txt
import idautils, idc
out = []
for line in open(r"D:\projects\project poland rewrite\research\phys_hr_addrs.txt"):
    line = line.strip()
    if not line:
        continue
    a = int(line, 16)
    for x in idautils.XrefsTo(a):
        out.append("%X -> %X %s | %s" % (a, x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
open(r"D:\projects\project poland rewrite\research\phys_hr_xref.txt", "w").write("\n".join(out))
