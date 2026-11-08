import idautils, idc
out = []
for ea in (0x29A120, 0x29A220, 0x29A310):
    for x in idautils.DataRefsTo(ea):
        out.append("%X -> %X" % (ea, x))
open(r"D:/projects/project poland rewrite/research/fe_cam_vt.txt", "w").write("\n".join(out))
