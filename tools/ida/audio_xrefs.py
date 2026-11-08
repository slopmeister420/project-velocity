import idautils, idc, ida_funcs, ida_bytes
# list functions referencing audio-related strings
import re
pat = re.compile(r'(?i)audio|\.xv|svg|_AUDIO|aee|MusLists|velocity\.vol|voice\.bin|Chain Structs')
out = {}
for s in idautils.Strings():
    t = str(s)
    if not pat.search(t):
        continue
    for x in idautils.XrefsTo(s.ea):
        f = ida_funcs.get_func(x.frm)
        if f:
            out.setdefault(f.start_ea, set()).add(t[:50])
for ea in sorted(out):
    print(hex(ea), idc.get_func_name(ea), '|', ' ; '.join(sorted(out[ea]))[:300])
