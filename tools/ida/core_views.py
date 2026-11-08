import idautils, idc, ida_funcs, ida_hexrays
out = []
done = set()
def dec(ea):
    f = ida_funcs.get_func(ea)
    if not f or f.start_ea in done:
        return
    done.add(f.start_ea)
    try:
        c = str(ida_hexrays.decompile(f.start_ea))
    except Exception as e:
        c = 'ERR %s' % e
    out.append('//==== %x %s\n%s\n' % (f.start_ea, idc.get_func_name(f.start_ea), c))
for s in [0x0063092c, 0x006309cc, 0x00630134]:
    for x in idautils.XrefsTo(s):
        out.append('// str %x ref at %x' % (s, x.frm))
        dec(x.frm)
open(r'D:/projects/project poland rewrite/research/core_views.c', 'w').write('\n'.join(out))
