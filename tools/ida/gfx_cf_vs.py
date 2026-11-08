# VertexStream append / begin / end + Renderer pretransformed draw -> research/gfx/cf_vs.c
import idc, ida_hexrays
out = []
for ea in (0x2B95D0, 0x2B8C70, 0x2B90A0, 0x2B87E0, 0x2B8810, 0x43FB0, 0x2A9B00, 0x24F90):
    try:
        out.append("//==== %X %s" % (ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
open(r"D:\projects\project poland rewrite\research\gfx\cf_vs.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
