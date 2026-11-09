import idautils, idc, ida_funcs
for a in (0x6F5C0C, 0x6F5C08, 0x6F5C24, 0x6F5C10):
    print("==", hex(a))
    seen=set()
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        n = idc.get_func_name(x.frm)
        if n in seen: continue
        seen.add(n)
        print(hex(x.frm), n)
