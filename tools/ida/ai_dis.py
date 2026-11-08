import idc, idautils
# disasm windows around addresses -> research/ai_dump/dis.txt
out = open(r"D:\projects\project poland rewrite\research\ai_dump\dis.txt", "w")
for ea, before, after in [(0x2D3E4, 60, 40), (0x2F3D3, 60, 40)]:
    out.write("==== %X\n" % ea)
    s = ea - before
    s = idc.next_head(idc.prev_head(s))
    x = s
    while x < ea + after:
        out.write("%X  %s\n" % (x, idc.GetDisasm(x)))
        x = idc.next_head(x)
# writes of 0x3F19999A / references to +0x1E8 in 0x36E00
f = 0x36E00
x = f
end = idc.get_func_attr(f, idc.FUNCATTR_END)
out.write("==== 36E00 lines with 1E8 or 6D3880\n")
while x < end:
    d = idc.GetDisasm(x)
    if "1E8" in d or "6D3880" in d or "3F19999A" in d.upper():
        out.write("%X  %s\n" % (x, d))
    x = idc.next_head(x)
out.close()
