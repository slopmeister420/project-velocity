import idautils, idc, ida_funcs
for a in [0x4E660,0x4ED80,0x4EBE0,0x4E8C0,0x4E9F0,0x4EB20,0x4F0A0,0x4F120,0x4FB50,0x4FBD0,0x49420,0x4A390,0x4A710,0x4B9C0,0x55F90,0x5A0 and 0x50670,0x506F0,0x4BE10,0x4B150,0x56A20]:
    cs = sorted(set(hex(ida_funcs.get_func(x).start_ea)+":"+(idc.get_func_name(x) or "") for x in idautils.CodeRefsTo(a,0) if ida_funcs.get_func(x)))
    ds = len(list(idautils.DataRefsTo(a)))
    print(hex(a), idc.get_func_name(a), "calls:", cs[:6], "data", ds)
