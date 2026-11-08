# car interior: who writes/reads FE vehicle loader +15694 (0x3D4E) -> research/gfx/int2.txt
import idc, idautils, ida_funcs, ida_hexrays
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
funcs = set()
for seg in idautils.Segments():
    if idc.get_segm_name(seg) not in (".text",):
        continue
    for f in idautils.Functions(seg, idc.get_segm_end(seg)):
        if f >= 0x3DDD20:
            continue
        for ea in idautils.FuncItems(f):
            for k in range(2):
                if idc.get_operand_type(ea, k) in (idc.o_displ, idc.o_phrase) and idc.get_operand_value(ea, k) == 0x3D4E:
                    out.append("%X %s | %s" % (ea, idc.get_func_name(ea), idc.generate_disasm_line(ea, 0)))
                    funcs.add(f)
open(B + r"\int2.txt", "w", encoding="utf-8").write("\n".join(out))
dec = []
for f in sorted(funcs):
    try:
        dec.append("//==== %X %s\n%s" % (f, idc.get_func_name(f), ida_hexrays.decompile(f)))
    except Exception as ex:
        dec.append("// fail %X %s" % (f, ex))
open(B + r"\int2.c", "w", encoding="utf-8").write("\n".join(dec))
print("ok", len(out))
