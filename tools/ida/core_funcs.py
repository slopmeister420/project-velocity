import idc, idautils
# function names in [start,end) from research/core_funcs.in -> research/core_funcs.txt (name, xref callers)
out = []
p = open(r"D:/projects/project poland rewrite/research/core_funcs.in").read().split()
s, e = int(p[0], 16), int(p[1], 16)
for f in idautils.Functions(s, e):
    callers = sorted(set(idc.get_func_name(x.frm) for x in idautils.XrefsTo(f)))
    out.append("%08X %s <- %s" % (f, idc.get_func_name(f), ", ".join(c for c in callers if c)[:200]))
open(r"D:/projects/project poland rewrite/research/core_funcs.txt", "w").write("\n".join(out))
