import idautils, idc, ida_funcs, ida_hexrays
out = []
# string xref -> function that asserts in SetViewport
targets = set()
for ea in idautils.Functions():
    n = idc.get_func_name(ea)
    if 'iewport' in n:
        targets.add(ea)
s = 0x00476c54
for x in idautils.XrefsTo(s):
    f = ida_funcs.get_func(x.frm)
    if f:
        targets.add(f.start_ea)
for t in sorted(targets):
    out.append('TARGET %x %s' % (t, idc.get_func_name(t)))
    for x in idautils.XrefsTo(t):
        f = ida_funcs.get_func(x.frm)
        if f and f.start_ea < 0x3DDD20:
            out.append('  caller %x in %x %s' % (x.frm, f.start_ea, idc.get_func_name(f.start_ea)))
open(r'D:/projects/project poland rewrite/research/core_viewport.txt', 'w').write('\n'.join(out))
