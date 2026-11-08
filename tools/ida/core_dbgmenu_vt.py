import idc
# vtable slots of the debug menus -> research/core_dbgmenu_vt.txt
VT = [0x62F0D0, 0x62F288, 0x62EA10, 0x62EC6C, 0x62EA5C]
out = []
for v in VT:
    out.append("== %X" % v)
    for i in range(12):
        t = idc.get_wide_dword(v + 4 * i)
        out.append("  +%d %X %s" % (4 * i, t, idc.get_func_name(t)))
open(r"D:\projects\project poland rewrite\research\core_dbgmenu_vt.txt", "w").write("\n".join(out))
print("ok")
