import ida_bytes, idc, struct
# bink xbox dsound driver volume constants (0x543617 fmul [0x673530], 0x543627 fmul [0x67352C]), fistp rounding mode
out = []
for a in (0x673530, 0x67352C):
    raw = ida_bytes.get_dword(a)
    out.append("%X = %08X %r" % (a, raw, struct.unpack("<f", struct.pack("<I", raw))[0]))
for ea in (0x543617, 0x54361B, 0x543621, 0x543623, 0x543625, 0x543627, 0x54362D):
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
open(r"D:\projects\project poland rewrite\research\audio\bink_vol.txt", "w").write("\n".join(out))
