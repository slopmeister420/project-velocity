# VsVector4 helpers' argument order -> research/gfx/tb_vec.c
import ida_hexrays, idc, ida_name
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x293D0, 0x13670]:
    n = idc.get_func_name(a)
    out.append("//==== %s %X  [%s]" % (n, a, idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\tb_vec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
