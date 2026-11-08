# physics agent: dump the expression parser's function and constant tables
import idc, ida_bytes, struct
nf = ida_bytes.get_dword(0x6CDB60)
nc = ida_bytes.get_dword(0x6CDB64)
print("functions", nf)
for i in range(nf):
    nm = ida_bytes.get_dword(0x6CDA30 + 12 * i)
    fn = ida_bytes.get_dword(0x6CDA34 + 12 * i)
    na = ida_bytes.get_dword(0x6CDA38 + 12 * i)
    print(" %-10s %08X args=%d" % (idc.get_strlit_contents(nm).decode(), fn, na))
print("constants", nc)
for i in range(nc):
    nm = ida_bytes.get_dword(0x6CDB50 + 8 * i)
    v = ida_bytes.get_dword(0x6CDB54 + 8 * i)
    print(" %-10s %08X %r" % (idc.get_strlit_contents(nm).decode(), v, struct.unpack("<f", struct.pack("<I", v))[0]))
