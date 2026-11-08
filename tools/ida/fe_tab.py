# frontend agent: dump string pointer table (fe_tab_args.txt: addr count)
import idc, ida_bytes
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_tab_args.txt").read().split()
ea, n = int(a[0], 16), int(a[1])
with open(r"D:\projects\project poland rewrite\research\fe_tab.txt", "w") as o:
    for i in range(n):
        p = ida_bytes.get_dword(ea + 4*i)
        s = idc.get_strlit_contents(p, -1, 0)
        o.write("%d %08X %s\n" % (i, p, s))
