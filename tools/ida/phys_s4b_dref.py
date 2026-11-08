# physics agent stage 4b: data references to a function (vtables holding it)
import idc, idautils
out = []
for a in [0x377B50]:
    for x in idautils.DataRefsTo(a):
        out.append("%X -> %X %s" % (x, a, idc.get_name(x)))
        for i in range(-8, 12):
            out.append("   +%d %X %s" % (4 * i, idc.get_wide_dword(x + 4 * i), idc.get_func_name(idc.get_wide_dword(x + 4 * i))))
open(r"D:\projects\project poland rewrite\research\phys_s4b_dref.txt", "w").write("\n".join(out))
