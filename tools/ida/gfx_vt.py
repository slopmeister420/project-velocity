import idc, idautils
for f in (0x2E1A70, 0x2E11C0, 0x2E0E40):
    for x in idautils.DataRefsTo(f):
        print('%X ref %X' % (f, x), ['%X' % idc.get_wide_dword(x - 4*k) for k in range(12, -1, -1)][-13:], ['%X' % idc.get_wide_dword(x + 4*k) for k in range(0, 12)])
