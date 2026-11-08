import idautils, ida_funcs, idc
for a in [7297576]:
    fs = []
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        fs.append('%x %s @%x' % (f.start_ea, idc.get_func_name(f.start_ea), x.frm) if f else '?@%x' % x.frm)
    print(hex(a), repr(idc.get_strlit_contents(a) or b'')[:60], '|', '; '.join(fs))
