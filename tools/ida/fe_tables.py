# frontend agent: dump the common overlay nav icon tables
import idc, ida_bytes, struct
def s(p):
    return idc.get_strlit_contents(p) if p else None
for i in range(15):
    x = struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(0x6C1638 + 8 * i)))[0]
    y = struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(0x6C163C + 8 * i)))[0]
    a = ida_bytes.get_dword(0x6C16B0 + 12 * i)
    b = ida_bytes.get_dword(0x6C16B4 + 12 * i)
    c = ida_bytes.get_dword(0x6C16B8 + 12 * i)
    print(i, x, y, s(a), s(b), s(c))
