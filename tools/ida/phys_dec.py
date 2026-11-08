# physics agent: decompile a few functions to research/_dec.c (addresses in research/phys_addrs.txt)
import ida_hexrays, ida_funcs
out = []
for line in open(r"D:\projects\project poland rewrite\research\phys_addrs.txt"):
    a = int(line.strip(), 16)
    out.append("// ==== %X\n%s" % (a, ida_hexrays.decompile(a)))
open(r"D:\projects\project poland rewrite\research\_dec.c", "w").write("\n".join(out))
