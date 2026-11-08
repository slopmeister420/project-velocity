# car interior: who loads \VEHICLEI.VCD -> research/gfx/int1.c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
for s in (0x6575C4,):
    for x in idautils.XrefsTo(s):
        f = ida_funcs.get_func(x.frm)
        out.append("// %X xref %X in %s" % (s, x.frm, idc.get_func_name(x.frm)))
        if f and f.start_ea not in seen:
            seen.add(f.start_ea)
            try:
                out.append(str(ida_hexrays.decompile(f.start_ea)))
            except Exception as ex:
                out.append("// fail %s" % ex)
open(B + r"\int1.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(seen))
