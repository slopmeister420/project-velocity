# decompile the addresses listed in research/gfx/el_dec_in.txt (hex, one per line; "#" comments) -> el_dec_out.c
import ida_hexrays, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_dec_in.txt"):
    l = l.split("#")[0].strip()
    if not l:
        continue
    a = int(l, 16)
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\el_dec_out.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
