import idc, idautils
# respawn review (ai agent): disasm of 0x20700 and xrefs of flt_684E78 -> research/ai_dump/respawn_dis.txt
out = open(r"D:\projects\project poland rewrite\research\ai_dump\respawn_dis.txt", "w")
for f in (0x20700,):
    x = f
    end = idc.get_func_attr(f, idc.FUNCATTR_END)
    while x < end:
        out.write("%X  %s\n" % (x, idc.GetDisasm(x)))
        x = idc.next_head(x)
for a in (0x684E78,):
    out.write("==== xrefs %X\n" % a)
    for r in idautils.XrefsTo(a):
        out.write("%X %s type %d  %s\n" % (r.frm, idc.get_func_name(r.frm), r.type, idc.GetDisasm(r.frm)))
out.close()
