# frontend agent: vtable dump (fe_tab_args.txt: addr count) -> research/fe_tab.txt
import idc, ida_bytes
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_tab_args.txt").read().split()
ea, n = int(a[0], 16), int(a[1])
with open(r"D:\projects\project poland rewrite\research\fe_tab.txt", "w") as o:
    for i in range(n):
        p = ida_bytes.get_dword(ea + 4*i)
        o.write("+%d %08X %s\n" % (4*i, p, idc.get_func_name(p)))
