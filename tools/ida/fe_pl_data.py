import idc
for a in (0x760220,0x760224,0x760228,0x76022C,0x760230,0x77FD3C):
    print(hex(a), hex(idc.get_wide_dword(a)), idc.get_name(a))
for a in (0x22C430,):
    pass
import ida_funcs
