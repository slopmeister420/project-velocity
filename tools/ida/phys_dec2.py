# physics agent: decompile the addresses in research/_dec_addrs.txt to research/_dec2.c
import ida_hexrays, ida_funcs
out = []
for line in open(r"D:\projects\project poland rewrite\research\_dec_addrs.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    a = int(line, 16)
    try:
        out.append("// ==== %X\n%s" % (a, ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// ==== %X fail %s" % (a, ex))
open(r"D:\projects\project poland rewrite\research\_dec2.c", "w").write("\n".join(out))
