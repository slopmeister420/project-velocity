# 0x12D2C0 -> sub_439FB0 (global matrix getter?) and writers of global matrix 0 -> research/gfx/envmap6_dec.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x439FB0, 0x12D090, 0x12C1B0, 0x12C210]:
    try:
        out.append("// ===== %X %s\n%s" % (ea, idc.get_func_name(ea), str(ida_hexrays.decompile(ea))))
    except Exception as ex:
        out.append("// fail %X %s" % (ea, ex))
callers = []
for x in idautils.XrefsTo(0x12D2C0):
    callers.append("%X %s" % (x.frm, idc.get_func_name(x.frm)))
out.append("// 12D2C0 callers:\n// " + "\n// ".join(callers))
open(B + r"\envmap6_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
