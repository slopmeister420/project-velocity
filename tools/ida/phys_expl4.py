# physics agent: explosion port - small vector / rand helpers
import ida_name, ida_hexrays, idc
ROOT = r"D:\projects\project poland rewrite\research"
out = []
for n in ["VsAdd", "VsSub", "VsRandRange", "VsRandUnit"]:
    a = idc.get_name_ea_simple(n)
    try:
        s = str(ida_hexrays.decompile(a, flags=ida_hexrays.DECOMP_NO_CACHE))
    except Exception as ex:
        s = "fail %s" % ex
    out.append("// ==== %X %s\n%s" % (a, n, s))
open(ROOT + r"\phys_expl4.txt", "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok")
