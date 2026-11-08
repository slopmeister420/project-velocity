import idautils, idc
# writes to setup colour d3dcolors (+0xBC/+0xC0) or the fe slot copy (+0x7DC/+0x7E0), and +34h/+38h pairs
out = []
for f in idautils.Functions(0x11000, 0x3DDD20):
    hits = set()
    for h in idautils.FuncItems(f):
        d = idc.generate_disasm_line(h, 0)
        if not d.startswith('mov'):
            continue
        for k in ('+0BCh],', '+0C0h],', '+7DCh],', '+7E0h],', '+34h],', '+38h],'):
            if k in d:
                hits.add(k)
    if ('+0BCh],' in hits and '+0C0h],' in hits) or ('+7DCh],' in hits) or ('+34h],' in hits and '+38h],' in hits):
        out.append('%X %s %s' % (f, idc.get_func_name(f), sorted(hits)))
open(r'D:/projects/project poland rewrite/research/gfx2/colfill2.txt', 'w').write('\n'.join(out))
print(len(out))
