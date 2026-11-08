import idc, idautils, ida_funcs
out = []
for a, b in [(0xb6050, 0xb60a8), (0xb5600, 0xb5630), (0xb8320, 0xb8380)]:
    ea = a
    while ea < b:
        out.append('%x %s' % (ea, idc.GetDisasm(ea)))
        ea = idc.next_head(ea)
    out.append('----')
# who references the tweak name string
for s in idautils.Strings():
    if str(s) == 'System Update Rate':
        for x in idautils.XrefsTo(s.ea):
            f = ida_funcs.get_func(x.frm)
            out.append('str ref %x %s' % (x.frm, idc.get_func_name(f.start_ea) if f else '?'))
open(r'D:/projects/project poland rewrite/research/core_rate4.txt', 'w').write('\n'.join(out))
