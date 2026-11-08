import idc, ida_hexrays, ida_funcs, ida_bytes
vts = {'base':0x665218,'int':0x6651C8,'float':0x665268,'string':0x6652B8,'bool':0x66B558,'struct':0x6708F0,'enum':0x670970,'array':0x6707C0,'vararray':0x670810}
for k,v in vts.items():
    print('#####', k, hex(v))
    for i in range(20):
        f = ida_bytes.get_dword(v+4*i)
        fn = ida_funcs.get_func(f)
        if not fn: break
        sz = fn.end_ea - fn.start_ea
        txt = ''
        if sz < 120:
            try:
                t = str(ida_hexrays.decompile(f)).split('\n')
                txt = ' '.join(l.strip() for l in t[1:] if l.strip() not in ('{','}') and 'debugbreak' not in l and 'off_6C18D8' not in l)[:300]
            except Exception as e:
                txt = 'ERR'
        print('  +%02d %x %s (%d) %s' % (4*i, f, idc.get_func_name(f), sz, txt))
