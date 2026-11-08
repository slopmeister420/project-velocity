# misc TrailBlazerRender globals -> research/gfx/tb_misc.txt
import idc, idautils, ida_funcs, ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x7374C0, 0x7187E0, 0x6F5BC8]:
    out.append("== xrefs %X" % ea)
    for x in list(idautils.XrefsTo(ea))[:12]:
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s  %s" % (x.frm, f and idc.get_func_name(f.start_ea), idc.generate_disasm_line(x.frm, 0)))
for lo, hi in [(0xB91C0, 0xB9200), (0xB9500, 0xB9530), (0xB9700, 0xB9730), (0xB82A0, 0xB82B8)]:
    out.append("== dis %X" % lo)
    ea = lo
    while ea < hi:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea, hi + 16)
for a in [0x97E90]:
    out.append(str(ida_hexrays.decompile(a)))
open(B + r"\tb_misc.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
