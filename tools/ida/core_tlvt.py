import idc, idautils, ida_bytes, ida_funcs
# per function in the timeline object range: strings pushed and vtables stored; then dump each vtable
out = []
vts = set()
for f in idautils.Functions(0x176B50, 0x17FEB0):
    strs = []; vt = []
    for h in idautils.FuncItems(f):
        d = idc.GetDisasm(h)
        for x in idautils.DataRefsFrom(h):
            s = idc.get_strlit_contents(x)
            if s and (s.startswith(b'F3') or s.startswith(b'VS') or s.startswith(b'Vs')):
                strs.append(s.decode())
            if 0x630000 <= x < 0x660000 and d.startswith('mov') and 'offset' in d and '[' in d.split(',')[0]:
                vt.append(x); vts.add(x)
    if strs or vt:
        out.append('%x %s strs=%s vt=%s' % (f, idc.get_func_name(f), strs, ['%x' % v for v in vt]))
for v in sorted(vts):
    ents = []
    for i in range(10):
        p = ida_bytes.get_dword(v + 4 * i)
        if not (0x11000 <= p < 0x3E0000):
            break
        ents.append('%x' % p)
        if i > 0 and idc.get_name(v + 4 * i).startswith('off_'):
            break
    out.append('VT %x: %s' % (v, ' '.join(ents)))
open(r'D:\projects\project poland rewrite\research\core_tlvt.txt', 'w').write('\n'.join(out))
print(len(out))
