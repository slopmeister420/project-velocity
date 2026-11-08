import idc, idautils
out = open(r"D:\projects\project poland rewrite\research\core_xref_str.txt", "w")
for s in (0x63b324, 0x63b32c):
    for x in idautils.XrefsTo(s):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("%x %x %s\n" % (s, x.frm, idc.get_func_name(f)))
out.close()
