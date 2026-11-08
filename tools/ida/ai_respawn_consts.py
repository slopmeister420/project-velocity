import idc, ida_bytes, struct
# respawn constants and helpers -> research/ai_dump/respawn_consts.txt
out = open(r"D:\projects\project poland rewrite\research\ai_dump\respawn_consts.txt", "w")
for a in [0x625754, 0x625750, 0x62550C, 0x624A90, 0x684E78]:
    v = ida_bytes.get_dword(a)
    out.write("%X  %08X  %r\n" % (a, v, struct.unpack("<f", struct.pack("<I", v))[0]))
for f in [0x248C0]:
    out.write("==== %X\n" % f)
    x = f
    end = idc.get_func_attr(f, idc.FUNCATTR_END)
    while x < end:
        out.write("%X  %s\n" % (x, idc.GetDisasm(x)))
        x = idc.next_head(x)
# who writes base +0x80 (velocity?) in VsAIVehicle update: grep 0x2C000..0x2D000 for "+80h]"
out.write("==== writes [*+80h] in 2B000..2D000\n")
x = 0x2B000
while x < 0x2D000:
    d = idc.GetDisasm(x)
    if "+80h]" in d and ("mov" in d or "fst" in d or "movss" in d):
        out.write("%X  %s  ; %s\n" % (x, d, idc.get_func_name(x)))
    x = idc.next_head(x)
out.close()
