import idc, idautils
# callers (two levels) of each graphics AddTweaks -> research/gfx/tweak_callers.txt
FUNCS = [0x6B8B0, 0x107560, 0x10D7E0, 0x10E3F0, 0x11E080, 0x1212D0, 0x122AE0, 0x123360, 0x1287B0, 0x131130,
         0x135120, 0x137610, 0x13D320, 0x1411C0, 0x145F20, 0x148BA0, 0x155070, 0x15A5C0, 0x15AF70, 0x15F820,
         0x161190]
def callers(ea):
    out = []
    for x in idautils.CodeRefsTo(ea, 0):
        f = idc.get_func_attr(x, idc.FUNCATTR_START)
        out.append((x, f, idc.get_func_name(f)))
    return out
o = open(r"D:\projects\project poland rewrite\research\gfx\tweak_callers.txt", "w")
for f in FUNCS:
    o.write("== %x %s\n" % (f, idc.get_func_name(f)))
    for x, c, n in callers(f):
        o.write("  %x in %x %s\n" % (x, c, n))
        for x2, c2, n2 in callers(c)[:6]:
            o.write("      %x in %x %s\n" % (x2, c2, n2))
o.close()
print("ok")
