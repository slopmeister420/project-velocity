# physics agent: functions referencing the string addresses in research/damage_dump/straddrs.txt
import os, idautils, ida_funcs, ida_name

ROOT = r"D:\projects\project poland rewrite\research\damage_dump"
with open(os.path.join(ROOT, "strxrefs.txt"), "w") as o:
    for a in open(os.path.join(ROOT, "straddrs.txt")).read().split():
        a = int(a, 16)
        for x in idautils.XrefsTo(a):
            f = ida_funcs.get_func(x.frm)
            o.write("%08X <- %08X %s\n" % (a, x.frm, ida_name.get_name(f.start_ea) if f else "?"))
print("ok")
