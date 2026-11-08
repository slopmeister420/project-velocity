# physics agent (handling review): decompile research/phys_hr_addrs.txt -> research/phys_hr_dec.c
import ida_hexrays
out = []
for line in open(r"D:\projects\project poland rewrite\research\phys_hr_addrs.txt"):
    line = line.strip()
    if not line:
        continue
    a = int(line, 16)
    try:
        out.append("// ==== %X\n%s" % (a, ida_hexrays.decompile(a)))
    except Exception as e:
        out.append("// ==== %X failed %s" % (a, e))
open(r"D:\projects\project poland rewrite\research\phys_hr_dec.c", "w").write("\n".join(out))
