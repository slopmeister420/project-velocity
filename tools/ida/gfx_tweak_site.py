import idc, idautils
o = open(r"D:\projects\project poland rewrite\research\gfx\tweak_site.txt", "w")
for t in [0x79C30, 0x143140, 0x18000, 0xB4170]:
    for x in idautils.CodeRefsTo(t, 0):
        f = idc.get_func_attr(x, idc.FUNCATTR_START)
        o.write("%x <- %x in %x %s\n" % (t, x, f, idc.get_func_name(f)))
o.close()
print("ok")
