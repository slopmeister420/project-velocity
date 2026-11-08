import idautils, idc, ida_funcs
OUT = 'D:/projects/project poland rewrite/research/core_s7_dis2.txt'
lines = []
for a in (0x167D60, 0x165DD0):
    f = ida_funcs.get_func(a)
    lines.append('==== %x %s' % (a, idc.get_func_name(a)))
    for h in idautils.Heads(f.start_ea, f.end_ea):
        lines.append('%x  %s' % (h, idc.GetDisasm(h)))
open(OUT, 'w').write('\n'.join(lines))
print(len(lines))
