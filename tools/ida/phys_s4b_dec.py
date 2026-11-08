# physics agent stage 4b: decompile the addresses in research/phys_s4b_list.txt to research/phys_s4b_dec.c
import ida_hexrays, idc
out = []
for line in open(r"D:\projects\project poland rewrite\research\phys_s4b_list.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    a = int(line, 16)
    try:
        out.append("// ==== %X %s\n%s" % (a, idc.get_func_name(a), ida_hexrays.decompile(a)))
    except Exception as e:
        out.append("// ==== %X failed %s" % (a, e))
open(r"D:\projects\project poland rewrite\research\phys_s4b_dec.c", "w", encoding="utf-8").write("\n".join(out))
print(len(out))
