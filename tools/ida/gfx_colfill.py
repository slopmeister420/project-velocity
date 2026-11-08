import idautils, idc, ida_funcs
# functions that read a +0x1C float triple and write +0x34 (colour record d3dcolor fill)
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    has1c = has34 = False
    for h in idautils.FuncItems(f):
        d = idc.generate_disasm_line(h, 0)
        if '+1Ch]' in d and ('movss' in d or 'fld' in d or 'cvt' in d):
            has1c = True
        if ('+34h],' in d) and d.startswith('mov'):
            has34 = True
    if has1c and has34:
        out.append('%X %s' % (f, idc.get_func_name(f)))
open(r'D:/projects/project poland rewrite/research/gfx2/colfill.txt', 'w').write('\n'.join(out))
print(len(out))
