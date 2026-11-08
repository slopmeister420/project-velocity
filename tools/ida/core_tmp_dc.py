import ida_hexrays, ida_funcs, idc, re
ADDRS = [0x11cdb0, 0x164d70, 0x1674e0, 0x165e00]
OUT = 'D:/projects/project poland rewrite/research/core_s7_dc1.c'

NOISE = re.compile(r"__debugbreak|^\s*[{}]\s*$|off_6C18D8 != nullptr|InputDevice__3DDC30\(1\);|memset\(.*0xCC|= off_6C18D8\(|if \( v[0-9]+ == 3 \)|(HI|LO)WORD\(v.*= 257;|BYTE[0-9]?\(v.*= 1;|^\s*$")
parts = []
for a in ADDRS:
    f = ida_funcs.get_func(a)
    parts.append('=' * 20 + ' %x %s' % (a, idc.get_func_name(a)))
    try:
        txt = str(ida_hexrays.decompile(f.start_ea))
        parts.extend(l for l in txt.split('\n') if not NOISE.search(l))
    except Exception as e:
        parts.append('ERR %s' % e)
text = '\n'.join(parts)
if OUT:
    open(OUT, 'w', encoding='utf-8').write(text)
    print('wrote', OUT, len(text))
else:
    print(text)
