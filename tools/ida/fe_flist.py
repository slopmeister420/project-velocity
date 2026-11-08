# frontend agent: list functions (start size name) in range fe_peek_args.txt -> research/fe_flist.txt
import idautils, idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
with open(r"D:\projects\project poland rewrite\research\fe_flist.txt", "w") as o:
    for f in idautils.Functions(int(a[0],16), int(a[1],16)):
        o.write("%08X %5X %s\n" % (f, idc.get_func_attr(f, idc.FUNCATTR_END)-f, idc.get_func_name(f)))
