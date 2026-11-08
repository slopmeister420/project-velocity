import idc
for a in (0x6C1510,0x6C1514,0x6C1518,0x624968):
    print("%X %08X %s" % (a, idc.get_wide_dword(a), idc.get_name(a)))
