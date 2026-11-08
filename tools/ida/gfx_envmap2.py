# dynamic envmap stage 2: object address at the init call, the whole function range around it,
# xrefs of the object -> research/gfx/envmap2_*.txt/c
import ida_hexrays, idc, ida_funcs, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
ea = 0x10E3CC
for i in range(8):
    ea = idc.prev_head(ea)
lines = []
e = ea
while e <= 0x10E3D8:
    lines.append("%X %s" % (e, idc.generate_disasm_line(e, 0)))
    e = idc.next_head(e)
out.append("\n".join(lines))
# functions 0x121840 .. 0x122FB0
e = 0x121840
funcs = []
while e < 0x122FB0:
    f = ida_funcs.get_func(e)
    if f is None:
        e = idc.next_head(e)
        continue
    funcs.append(f.start_ea)
    e = f.end_ea
for s in funcs:
    f = ida_funcs.get_func(s)
    callers = ["%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(s)]
    out.append("// ===== %X..%X %s callers: %s" % (f.start_ea, f.end_ea, idc.get_func_name(s), " ".join(callers[:12])))
open(B + r"\envmap2_funcs.txt", "w", encoding="utf-8").write("\n".join(out))
dec = []
for s in funcs:
    try:
        dec.append("// ===== %X %s\n%s" % (s, idc.get_func_name(s), str(ida_hexrays.decompile(s))))
    except Exception as ex:
        dec.append("// fail %X %s" % (s, ex))
open(B + r"\envmap2_dec.c", "w", encoding="utf-8").write("\n".join(dec))
print("ok", len(funcs))
