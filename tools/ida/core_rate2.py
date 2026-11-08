import idautils, idc, ida_funcs, ida_hexrays
out = []
for x in idautils.XrefsTo(0x0068580C):
    f = ida_funcs.get_func(x.frm)
    out.append('%x %s %s | %s' % (x.frm, idc.get_func_name(f.start_ea) if f else '?', x.type, idc.GetDisasm(x.frm)))
# VsBounds__57450 (rate setter?) and its callers
for name in ['VsBounds__57450']:
    ea = idc.get_name_ea_simple(name)
    out.append('//==== %x %s\n%s' % (ea, name, ida_hexrays.decompile(ea)))
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.append('  caller %x %s' % (x.frm, idc.get_func_name(f.start_ea) if f else '?'))
open(r'D:/projects/project poland rewrite/research/core_rate2.c', 'w').write('\n'.join(out))
