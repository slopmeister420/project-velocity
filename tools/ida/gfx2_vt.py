# dump vtables containing the function in FN: research/g2_vt.txt
import idautils, idc, ida_funcs
FN = [0x174A70]
out = open(r"D:\projects\project poland rewrite\research\g2_vt.txt", "w")
for fn in FN:
    for x in idautils.XrefsTo(fn):
        if ida_funcs.get_func(x.frm):
            continue
        out.write("ref %08X\n" % x.frm)
        ea = x.frm - 4 * 12
        for i in range(30):
            v = idc.get_wide_dword(ea + 4 * i)
            out.write("  %08X [%d] %08X %s\n" % (ea + 4 * i, (ea + 4 * i - x.frm) // 4, v, idc.get_func_name(v)))
out.close()
print("ok")
