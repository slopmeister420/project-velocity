# physics agent: explosion port - decal list users, static init of unk_7805B0
import idautils, ida_funcs, ida_name, ida_hexrays
ROOT = r"D:\projects\project poland rewrite\research"
out = []
for a in [0x162400, 0x133800, 0x431EFE, 0x4259B3, 0x1E2153]:
    f = ida_funcs.get_func(a)
    s = "nofunc"
    if f:
        try:
            s = str(ida_hexrays.decompile(f.start_ea, flags=ida_hexrays.DECOMP_NO_CACHE))
        except Exception as ex:
            s = "fail %s" % ex
    out.append("// ==== %X %s\n%s" % (a, ida_name.get_name(f.start_ea) if f else "?", s[:6000]))
open(ROOT + r"\phys_expl3.txt", "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok")
