# dump dwords (hex, float, name) for address ranges listed in research/ai_dump/_data.txt "lo hi" (ai agent)
import struct, idc, ida_bytes, ida_name
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_data.txt"):
    p = line.split()
    if len(p) < 2:
        continue
    lo, hi = int(p[0], 16), int(p[1], 16)
    for ea in range(lo, hi, 4):
        v = ida_bytes.get_dword(ea)
        f = struct.unpack("<f", struct.pack("<I", v))[0]
        n = ida_name.get_name(ea) or ""
        print("%08X %08X %-14.8g %s" % (ea, v, f, n))
