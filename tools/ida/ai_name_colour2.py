import idc, ida_hexrays
# name colour helpers -> research/ai_dump/name_colour2.c
out = open(r"D:\projects\project poland rewrite\research\ai_dump\name_colour2.c", "w")
for ea in [0x13C50, 0x13B60, 0x13BB0, 0x13C00, 0x15EB0, 0x15E50, 0x15E70, 0x15E90, 0x13F50, 0x13940]:
    out.write("==== %X %s\n" % (ea, idc.get_func_name(ea)))
    try:
        out.write(str(ida_hexrays.decompile(ea)) + "\n")
    except Exception as e:
        out.write("ERR %s\n" % e)
out.write("==== asm before 14416\n")
x = idc.prev_head(0x14416)
lines = []
for i in range(14):
    lines.append("%X  %s" % (x, idc.GetDisasm(x)))
    x = idc.prev_head(x)
out.write("\n".join(reversed(lines)) + "\n")
out.write("%X  %s\n" % (0x14416, idc.GetDisasm(0x14416)))
out.close()
