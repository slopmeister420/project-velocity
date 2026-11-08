# CarColour users -> research/gfx/cs_dec7.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for s in (0x63E660, 0x653B94):
    for x in idautils.XrefsTo(s):
        out.append("// str %X xref %X in %s : %s" % (s, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
f = ida_funcs.get_func(0x156C00)
ea = f.start_ea
while ea < f.end_ea:
    l = idc.generate_disasm_line(ea, 0)
    if "arg_4" in l or "156150" in l or "2A1A00" in l or "2A1990" in l or "2B1370" in l:
        out.append("// 156C00 %X %s" % (ea, l))
    ea = idc.next_head(ea)
f2 = ida_funcs.get_func(0x156C00)
out.append("// 156C00 range %X..%X" % (f2.start_ea, f2.end_ea))
for x in idautils.XrefsTo(0x653B94):
    try:
        out.append(str(ida_hexrays.decompile(x.frm)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\cs_dec7.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
