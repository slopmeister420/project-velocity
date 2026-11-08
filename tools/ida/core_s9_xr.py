import idautils, idc, ida_funcs
base=0x6FC098+2936
for ea in [base, 0x6FC098]:
    print(hex(ea))
    for x in idautils.XrefsTo(ea):
        f=ida_funcs.get_func(x.frm)
        print(' ', hex(x.frm), idc.get_func_name(x.frm))
