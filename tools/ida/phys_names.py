# physics agent: addresses of named globals listed in research/phys_names.txt -> research/phys_names_out.txt
import idc, ida_name
ROOT = r"D:\projects\project poland rewrite\research"
out = []
for n in open(ROOT + r"\phys_names.txt").read().split():
    out.append("%s %X" % (n, ida_name.get_name_ea(idc.BADADDR, n)))
open(ROOT + r"\phys_names_out.txt", "w").write("\n".join(out) + "\n")
print("\n".join(out))
