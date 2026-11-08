# bonnet smoke helpers -> research/gfx/bs_dec.c; callers of elParticleSystem start 0x2CF100 in CarDamage
import ida_hexrays, idc, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for a in [0x79170, 0x79190, 0x78040, 0x77F30, 0x77F50, 0x79120, 0x784A0, 0x76730, 0x766F0, 0x76970, 0x23DD0,
          0x2ABF70, 0x2ABBC0, 0x24870, 0x1C66C0, 0x78140]:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
out.append("//==== callers of 2CF100 / 2CF170 below 0x80000")
for t in (0x2CF100, 0x2CF170):
    for x in idautils.CodeRefsTo(t, 0):
        if x < 0x80000:
            out.append("// %X -> %X in %s" % (x, t, idc.get_func_name(x)))
open(B + r"\bs_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
