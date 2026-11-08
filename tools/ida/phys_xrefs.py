import idautils, idc
for a in (0x19C230, 0x19C390):
    print(hex(a), [ (hex(x.frm), idc.get_func_name(x.frm)) for x in idautils.XrefsTo(a)])
