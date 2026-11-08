# call-site context of the danger particle object byte_719168 -> research/gfx/dp_ctx.txt
import idc, ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for c in [0xB9DDF, 0xC567F, 0xC60BD, 0xC6104, 0x17A77A, 0x180640]:
    out.append("== %X" % c)
    ea = c - 0x30
    while ea < c + 0x18:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea, c + 0x40)
for a in [0x123450, 0x2D57D0, 0x2D5A10]:
    out.append("//==== %08X %s" % (a, idc.get_func_name(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
# B9010 section with the three calls
out.append("== B9DA0..B9E40")
ea = 0xB9DA0
while ea < 0xB9E40:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0xB9E50)
open(B + r"\dp_ctx.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
