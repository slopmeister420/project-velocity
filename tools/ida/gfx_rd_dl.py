# DrawList.cpp functions + callers of the add -> research/gfx/rd_dl.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x2C1FF0, 0x2C2360, 0x2C23A0, 0x2C23C0, 0x2C2440, 0x2C2460, 0x2C2650, 0x2C2A00, 0x2C2D10, 0x2C2D40,
          0x14D6C0, 0x2A9060]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
for t in (0x2C2360, 0x2C23A0, 0x2C23C0, 0x2C2440, 0x2C2460, 0x2C2650, 0x2C2D10, 0x2C2D40):
    for x in idautils.CodeRefsTo(t, 0):
        out.append("// %X -> %X in %s" % (x, t, idc.get_func_name(x)))
open(B + r"\rd_dl.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
