# physics agent: callers of the rpm/gear functions -> research/phys_gear_xrefs.txt
import idautils, idc
out = []
for t in (0x1CABF0,):
    out.append("== %X" % t)
    for x in idautils.CodeRefsTo(t, 0):
        out.append("%X %s" % (x, idc.get_func_name(x)))
open(r"D:\projects\project poland rewrite\research\phys_gear_xrefs.txt", "w").write("\n".join(out))
print("ok")
