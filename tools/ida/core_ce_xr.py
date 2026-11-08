import idautils, idc, ida_funcs
OUT = 'D:\\projects\\project poland rewrite\\research\\core_s7_ce_xr.txt'
lines = []
for f in idautils.Functions(0x11cdb0, 0x11e9f0):
    callers = sorted(set(hex(x.frm) + ':' + str(idc.get_func_name(x.frm)) for x in idautils.XrefsTo(f)))
    lines.append('%x %s <- %s' % (f, idc.get_func_name(f), ' '.join(callers[:8])))
for a in (0x164D70, 0x1674E0):
    callers = sorted(set(hex(x.frm) + ':' + str(idc.get_func_name(x.frm)) for x in idautils.XrefsTo(a)))
    lines.append('%x %s <- %s' % (a, idc.get_func_name(a), ' '.join(callers[:8])))
open(OUT, 'w').write('\n'.join(lines))
print(len(lines))
