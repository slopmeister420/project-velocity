# wheel blur: 110220 shader lines, 1134A0 context, constants -> research/gfx/wblur_dec4.c
import ida_hexrays, idc, idautils, ida_bytes, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
lines = str(ida_hexrays.decompile(0x110220)).split("\n")
out.append("//==== 110220 lines 270-300, 500-560")
for i in list(range(270, 300)) + list(range(500, 560)):
    if i < len(lines):
        out.append("%5d %s" % (i, lines[i]))
l2 = str(ida_hexrays.decompile(0x1134A0)).split("\n")
out.append("//==== 1134A0 (%d lines), lines near 116E30 and mask use" % len(l2))
for i, l in enumerate(l2):
    if "116E30" in l or "1024" in l or "0x800" in l or "2260" in l or "2264" in l:
        for j in range(max(0, i - 6), min(len(l2), i + 8)):
            out.append("%5d %s" % (j, l2[j]))
for a in list(range(0x6867D0, 0x6867F8, 4)) + [0x686610, 0x686614, 0x686618, 0x68661C, 0x7184E0]:
    d = ida_bytes.get_dword(a)
    out.append("// %X = %08X %r" % (a, d, struct.unpack("<f", struct.pack("<I", d))[0]))
    for x in idautils.DataRefsTo(a):
        out.append("//    <- %X %s" % (x, idc.get_func_name(x)))
open(B + r"\wblur_dec4.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
