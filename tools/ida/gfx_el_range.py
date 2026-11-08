# decompile every function in the ranges of research/gfx/el_range_in.txt ("start end") -> el_range_out.c
import idautils, ida_hexrays, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_range_in.txt"):
    p = l.split("#")[0].split()
    if len(p) < 2:
        continue
    for fa in idautils.Functions(int(p[0], 16), int(p[1], 16)):
        out.append("//==== %08X %s" % (fa, idc.get_func_name(fa)))
        try:
            out.append(str(ida_hexrays.decompile(fa)))
        except Exception as ex:
            out.append("// fail %s" % ex)
open(B + r"\el_range_out.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
