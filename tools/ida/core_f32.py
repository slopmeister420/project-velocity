import ida_bytes, struct
for ea in [0x685094, 0x685090]:
    v = ida_bytes.get_dword(ea)
    print("%X %08X %r" % (ea, v, struct.unpack("<f", struct.pack("<I", v))[0]))
