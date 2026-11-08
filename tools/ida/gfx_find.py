import idc,struct
for a in (0x685064,):
    print(hex(a), struct.unpack('<f', idc.get_bytes(a,4))[0], hex(idc.get_wide_dword(a)))
