import idautils, idc, ida_funcs, ida_hexrays
out = []
for ea in [0x00057250]:
    out.append('//==== %x\n%s' % (ea, ida_hexrays.decompile(ea)))
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.append('  caller %x %s' % (x.frm, idc.get_func_name(f.start_ea) if f else '?'))
        if f:
            c = str(ida_hexrays.decompile(f.start_ea)).split('\n')
            for i, l in enumerate(c):
                if '57250' in l:
                    out.extend(c[max(0, i - 10):i + 3])
open(r'D:/projects/project poland rewrite/research/core_rate3.c', 'w').write('\n'.join(out))
