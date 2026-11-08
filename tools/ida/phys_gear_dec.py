# physics agent: decompile the rpm/gear readers (HUD, audio) -> research/phys_gear_dec2.c
import ida_hexrays
A = [0x523C0, 0x11ABB0, 0x1BA770]
out = []
for a in A:
    try:
        out.append("// ==== %X\n%s" % (a, ida_hexrays.decompile(a)))
    except Exception as e:
        out.append("// ==== %X failed %s" % (a, e))
open(r"D:\projects\project poland rewrite\research\phys_gear_dec2.c", "w").write("\n".join(out))
print("ok")
