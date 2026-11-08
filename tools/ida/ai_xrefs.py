import idc, idautils, ida_funcs
for a in (0x34DB0, 0x40AC0, 0x2C720, 0x3DA60, 0x16FF0, 0x15A00):
    refs = []
    for x in idautils.XrefsTo(a, 0):
        f = ida_funcs.get_func(x.frm)
        refs.append("%X(%s)" % (x.frm, idc.get_func_name(f.start_ea) if f else "data"))
    print("%X %s <- %s" % (a, idc.get_func_name(a), ", ".join(refs[:6])))
