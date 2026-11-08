import idc, idautils, ida_funcs
# code xrefs to each address in research/core_xrefs.in (hex per line) -> research/core_xrefs.txt
out = []
for line in open(r"D:/projects/project poland rewrite/research/core_xrefs.in"):
    p = line.split()
    if not p:
        continue
    a = int(p[0], 16)
    out.append("== %X %s" % (a, idc.get_func_name(a)))
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        out.append("  %08X in %s" % (x.frm, idc.get_func_name(x.frm) if f else "?"))
open(r"D:/projects/project poland rewrite/research/core_xrefs.txt", "w").write("\n".join(out))
