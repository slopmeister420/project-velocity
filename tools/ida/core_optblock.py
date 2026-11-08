import idautils, idc, ida_funcs, ida_bytes
out = open(r"D:/projects/project poland rewrite/research/core_optblock.txt", "w")
def dump(lo, hi):
    ea = lo
    while ea < hi and ea != idc.BADADDR:
        refs = []
        for x in idautils.XrefsTo(ea):
            f = ida_funcs.get_func(x.frm)
            refs.append("%s@%X" % (idc.get_func_name(f.start_ea) if f else "?", x.frm))
        out.write("%X +%X %s size=%d refs=%d %s\n" % (ea, ea - lo, idc.get_name(ea), ida_bytes.get_item_size(ea), len(refs), " ".join(refs[:12])))
        ea = idc.next_head(ea, hi)
dump(0x6F915C, 0x6F9BDC)
out.write("---- 6D45C1\n")
dump(0x6D45C1, 0x6D45C2)
out.close()
