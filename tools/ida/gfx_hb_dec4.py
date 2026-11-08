# VsRender light overlap helpers of 0x5A040 -> research/gfx/hb_dec4.c (+ disasm of 0x5A040)
import ida_hexrays, idc, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x5A6C0, 0x59FB0, 0x5B200]:
    f = ida_funcs.get_func(ea)
    out.append("//==== %08X %s" % (f.start_ea, idc.get_func_name(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
for fea in [0x5A040, 0x59FB0]:
    f = ida_funcs.get_func(fea)
    out.append("//==== disasm %08X" % f.start_ea)
    ea = f.start_ea
    while ea < f.end_ea:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
open(B + r"\hb_dec4.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
