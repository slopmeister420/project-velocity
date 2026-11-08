import idc, idautils
# readers / writers of the audio tweak globals -> research/audio/tweak_xrefs.txt
VARS = [0x684F00, 0x684F04, 0x684F94, 0x684FB4, 0x684ED4, 0x684ED8, 0x684EF0, 0x684EC4, 0x684EC8]
o = open(r"D:\projects\project poland rewrite\research\audio\tweak_xrefs.txt", "w")
for v in VARS:
    o.write("== %x\n" % v)
    for x in idautils.DataRefsTo(v):
        f = idc.get_func_attr(x, idc.FUNCATTR_START)
        o.write("  %x in %x %s : %s\n" % (x, f if f != idc.BADADDR else 0, idc.get_func_name(x), idc.GetDisasm(x)))
o.close()
print("ok")
