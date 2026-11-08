import idautils, idc, ida_funcs, ida_hexrays, ida_lines
out = []
for a in [0x0016E17C, 0x000B38AC]:
    f = ida_funcs.get_func(a)
    out.append('//==== ref %x in %x %s' % (a, f.start_ea, idc.get_func_name(f.start_ea)))
    c = str(ida_hexrays.decompile(f.start_ea))
    lines = c.split('\n')
    for i, l in enumerate(lines):
        if '68580C' in l:
            out.extend(lines[max(0, i - 12):i + 8])
            out.append('----')
for name in ['VsBounds__572A0']:
    ea = idc.get_name_ea_simple(name)
    if ea != idc.BADADDR:
        out.append('//==== %x %s\n%s' % (ea, name, ida_hexrays.decompile(ea)))
open(r'D:/projects/project poland rewrite/research/core_rate.c', 'w').write('\n'.join(out))
