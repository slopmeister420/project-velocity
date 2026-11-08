# physics agent: write the functions in research/_ret_float.txt plus all their callers to research/phys_addrs.txt
import idautils, ida_funcs
R = r"D:\projects\project poland rewrite\research"
out = set()
for line in open(R + r"\_ret_float.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    f = int(line, 16)
    out.add(f)
    for x in idautils.CodeRefsTo(f, 0):
        g = ida_funcs.get_func(x)
        if g:
            out.add(g.start_ea)
open(R + r"\phys_addrs.txt", "w").write("\n".join("%X" % a for a in sorted(out)) + "\n")
print(len(out))
