import idc, idautils
# callers of the VE+4392 writer 0x1C2FF0 and the shake setter's caller -> research/phys_camscan2.txt
out = []
for t in (0x1C2FF0, 0x17A5F0, 0x1DDAE0):
    out.append("== callers of %X %s" % (t, idc.get_func_name(t)))
    for x in idautils.CodeRefsTo(t, 0):
        out.append("%08X %s" % (x, idc.get_func_name(x)))
        for y in idautils.CodeRefsTo(idc.get_func_attr(x, idc.FUNCATTR_START), 0):
            out.append("    <- %08X %s" % (y, idc.get_func_name(y)))
    for x in idautils.DataRefsTo(t):
        out.append("data %08X" % x)
open(r"D:/projects/project poland rewrite/research/phys_camscan2.txt", "w").write("\n".join(out))
