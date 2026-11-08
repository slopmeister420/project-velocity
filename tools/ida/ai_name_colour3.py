import idc, ida_bytes, idautils
# gradient colours and getter offsets -> research/ai_dump/name_colour3.txt
out = open(r"D:\projects\project poland rewrite\research\ai_dump\name_colour3.txt", "w")
for a in [0x77FD8C, 0x77FD84, 0x77FDA8, 0x77FD9C]:
    out.write("%X  %08X  seg %s\n" % (a, ida_bytes.get_dword(a), idc.get_segm_name(a)))
    for x in idautils.XrefsTo(a):
        out.write("   xref %X %s : %s\n" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
for f in [0x13C80, 0x13B90, 0x13BE0, 0x13C30]:
    x = f
    end = idc.get_func_attr(f, idc.FUNCATTR_END)
    out.write("==== %X\n" % f)
    while x < end:
        d = idc.GetDisasm(x)
        if "add" in d or "lea" in d:
            out.write("%X  %s\n" % (x, d))
        x = idc.next_head(x)
out.close()
