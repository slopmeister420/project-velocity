# frontend agent (tips stage): disassembly of the ranges in fe_tips_dis_args.txt -> research/fe_tips_dis.txt
import idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_tips_dis_args.txt").read().split()
with open(r"D:\projects\project poland rewrite\research\fe_tips_dis.txt", "w") as o:
    for i in range(0, len(a), 2):
        ea, end = int(a[i], 16), int(a[i + 1], 16)
        while ea < end:
            o.write("%08X %s\n" % (ea, idc.GetDisasm(ea)))
            ea = idc.next_head(ea)
