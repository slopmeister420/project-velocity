import idc, idautils
o = open(r"D:\projects\project poland rewrite\research\gfx\segments.txt", "w")
for s in idautils.Segments():
    o.write("%x %x %s\n" % (s, idc.get_segm_end(s), idc.get_segm_name(s)))
o.close()
print("ok")
