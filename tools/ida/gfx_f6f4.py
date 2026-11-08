import idautils, idc
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    for h in idautils.FuncItems(f):
        d = idc.generate_disasm_line(h, 0)
        if d.startswith('mov') and '+6F4h],' in d:
            out.append('%X %X %s | %s' % (f, h, idc.get_func_name(f), d))
open(r'D:/projects/project poland rewrite/research/gfx2/f6f4.txt', 'w').write('\n'.join(out))
print(len(out))
