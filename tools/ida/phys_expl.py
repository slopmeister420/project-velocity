# physics agent: explosion port - decompile (no cache) + callers of a fixed list into research/phys_expl.c
import idautils, ida_funcs, ida_name, ida_hexrays
ROOT = r"D:\projects\project poland rewrite\research"
dec = [0x17ACD0, 0x19FD80, 0x1E18E0, 0x1E1A20, 0x1B3500, 0x1B3850, 0x1E1F20, 0x2B0C20, 0x133A80, 0x133860,
       0x133750, 0x762F0, 0x19D310, 0x24CC0, 0x1D1170, 0x19DD0, 0x1A0510]
xr = [0x19FD80, 0x1E18E0, 0x1B3500, 0x1B3850, 0x1E1F20, 0x17ACD0, 0x1D1170]
out = []
for a in xr:
    out.append("== callers of %X %s" % (a, ida_name.get_name(a)))
    for x in idautils.CodeRefsTo(a, 0):
        f = ida_funcs.get_func(x)
        out.append("  %X in %X %s" % (x, f.start_ea if f else 0, ida_name.get_name(f.start_ea) if f else "?"))
for a in dec:
    try:
        s = str(ida_hexrays.decompile(a, flags=ida_hexrays.DECOMP_NO_CACHE))
    except Exception as ex:
        s = "fail %s" % ex
    out.append("// ==== %08X %s\n%s" % (a, ida_name.get_name(a), s))
open(ROOT + r"\phys_expl.c", "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok", len(out))
