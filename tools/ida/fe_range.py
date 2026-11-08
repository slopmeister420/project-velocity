# frontend agent: disassemble an address range into research/fe_range.txt (fe_peek_args.txt: start end)
import idautils, idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
s, e = int(a[0], 16), int(a[1], 16)
with open(r"D:\projects\project poland rewrite\research\fe_range.txt", "w") as o:
    for h in idautils.Heads(s, e):
        o.write("%08X  %s\n" % (h, idc.GetDisasm(h)))
print("ok")
