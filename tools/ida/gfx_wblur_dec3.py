# wheel blur level + shaders -> research/gfx/wblur_dec3.c
import ida_hexrays, idc, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in (0x116E30,):
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    out.append(str(ida_hexrays.decompile(a)))
    out.append("// callers: " + " ".join("%X(%s)" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(a, 0)))
f = ida_hexrays.decompile(0x110220)
lines = str(f).split("\n")
out.append("//==== 110220 lines mentioning +32/+36 / 0x20 / 0x24 / wblur / 1980 / 1996")
for i, l in enumerate(lines):
    if any(k in l for k in ("+ 32)", "+ 36)", "wblur", "117980", "SetBlendMode", "SetZState", "CreateShader", "SetTexture(", "2A2D60", "2A1A00", "2A1990", "Colour", "Alpha", "StageArgs", "2A1240", "2A10C0", "2A1140", "2A12C0", "2A0F40", "2A0FC0")):
        out.append("%5d %s" % (i, l))
open(B + r"\wblur_dec3.c", "w", encoding="utf-8").write("\n".join(out))
f2 = ida_hexrays.decompile(0x112A50)
l2 = str(f2).split("\n")
o2 = [l for l in l2]
open(B + r"\wblur_112a50.c", "w", encoding="utf-8").write("\n".join(o2))
print("ok")
