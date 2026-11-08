# physics agent: explosion port - who owns dword_720840 / unk_7805B0, decompile the 133750 helpers
import idautils, ida_funcs, ida_name, ida_hexrays, ida_bytes, struct
ROOT = r"D:\projects\project poland rewrite\research"
out = []
for a in [0x720840, 0x7805B0, 0x726388, 0x7814E0]:
    out.append("== data refs to %X %s" % (a, ida_name.get_name(a)))
    n = 0
    for x in idautils.DataRefsTo(a):
        f = ida_funcs.get_func(x)
        out.append("  %X in %X %s" % (x, f.start_ea if f else 0, ida_name.get_name(f.start_ea) if f else "?"))
        n += 1
        if n > 25:
            break
out.append("7805B0 words: " + " ".join("%08X" % ida_bytes.get_dword(0x7805B0 + 4 * k) for k in range(4)))
for a in [0x1337C0, 0x133A20, 0x133630]:
    try:
        s = str(ida_hexrays.decompile(a, flags=ida_hexrays.DECOMP_NO_CACHE))
    except Exception as ex:
        s = "fail %s" % ex
    out.append("// ==== %08X %s\n%s" % (a, ida_name.get_name(a), s))
open(ROOT + r"\phys_expl2.txt", "w", encoding="utf-8").write("\n".join(out) + "\n")
print("ok")
