# 0x15C3C0 quad uv + 0x29F7F0 texture size -> research/gfx/envmap7_dec.c
import ida_hexrays, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x15C3C0, 0x29F7F0, 0x15C240]:
    try:
        out.append("// ===== %X %s\n%s" % (ea, idc.get_func_name(ea), str(ida_hexrays.decompile(ea))))
    except Exception as ex:
        out.append("// fail %X %s" % (ea, ex))
open(B + r"\envmap7_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
