import idautils, idc, ida_funcs
for ea in [0x62dacc,0x62daf8,0x62d96c,0x64b41c,0x64b3f0,0x62c030,0x62bf2c,0x62c148]:
    for x in idautils.XrefsTo(ea):
        f=ida_funcs.get_func(x.frm)
        print(hex(ea), idc.get_strlit_contents(ea), hex(x.frm), f and idc.get_func_name(f.start_ea))
