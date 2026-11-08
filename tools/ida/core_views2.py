import idautils, idc, ida_funcs, ida_hexrays
out = []
seen = set()
def dec(ea, depth):
    if ea in seen or depth < 0:
        return
    seen.add(ea)
    try:
        c = str(ida_hexrays.decompile(ea))
    except Exception as e:
        c = 'ERR %s' % e
    out.append('//==== %x %s\n%s\n' % (ea, idc.get_func_name(ea), c))
    f = ida_funcs.get_func(ea)
    for h in idautils.FuncItems(ea):
        for x in idautils.CodeRefsFrom(h, False):
            g = ida_funcs.get_func(x)
            if g and g.start_ea == x and x < 0x3DDD20 and x not in seen:
                dec(x, depth - 1)
dec(0x000B3EC0, 1)
open(r'D:/projects/project poland rewrite/research/core_views2.c', 'w').write('\n'.join(out))
