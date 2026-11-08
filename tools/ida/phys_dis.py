import idautils, idc, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/phys_colour_dis3.txt", "w")
for a in (0x16580,):
    f = ida_funcs.get_func(a)
    for h in idautils.Heads(f.start_ea, f.end_ea):
        out.write("%x %s\n" % (h, idc.GetDisasm(h)))
