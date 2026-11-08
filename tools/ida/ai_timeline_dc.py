import idc, ida_hexrays, ida_bytes, struct
# TimelineAI respawn math -> research/ai_dump/timeline_ai_dc.c
out = open(r"D:\projects\project poland rewrite\research\ai_dump\timeline_ai_dc.c", "w")
for a in [0x625544]:
    v = ida_bytes.get_dword(a)
    out.write("%X  %08X  %r\n" % (a, v, struct.unpack("<f", struct.pack("<I", v))[0]))
for ea in [0x2B7A30, 0x23D30, 0x23D50, 0x17CF0]:
    out.write("==== %X %s\n" % (ea, idc.get_func_name(ea)))
    try:
        out.write(str(ida_hexrays.decompile(ea)) + "\n")
    except Exception as e:
        out.write("ERR %s\n" % e)
out.write("==== asm 177150 window around 625544\n")
x = 0x177150
end = idc.get_func_attr(x, idc.FUNCATTR_END)
lines = []
while x < end:
    lines.append("%X  %s" % (x, idc.GetDisasm(x)))
    x = idc.next_head(x)
for i, l in enumerate(lines):
    if "625544" in l:
        out.write("\n".join(lines[max(0, i - 14):i + 30]) + "\n")
out.close()
