# wheel blur helpers -> research/gfx/wblur_dec2.c; writers of car data +0x8D4/+0x8D8/+0x8DC, +0x7C8/+0x7D8 in game code
import ida_hexrays, idc, idautils, idaapi, ida_ua
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in (0x117980, 0x12B2D0, 0x2A05D0, 0x761E0):
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
    out.append("// callers: " + " ".join("%X(%s)" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(a, 0)))
out.append("//==== 110220 around 'wblur' (0x111600..0x111720)")
ea = 0x111600
while ea < 0x111720:
    out.append("// %X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x111720)
out.append("//==== stores to +8D4h/+8D8h/+8DCh/+7C8h/+7D8h/+7BCh/+7CCh in 0x11000..0x3DDD20")
ea = 0x11000
keys = ("+8D4h]", "+8D8h]", "+8DCh]", "+7C8h]", "+7D8h]", "+7BCh]", "+7CCh]")
while ea < 0x3DDD20 and ea != idc.BADADDR:
    m = idc.print_insn_mnem(ea)
    if m in ("mov", "movss", "fstp", "fst", "movd"):
        op0 = idc.print_operand(ea, 0)
        if any(k in op0 for k in keys):
            out.append("// %X %s | %s" % (ea, idc.get_func_name(ea), idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x3DDD20)
open(B + r"\wblur_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
