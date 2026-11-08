# callers outside elParticle (0x2C5A60..0x2D2E60, 0x317890..0x3184A0, 0x343290..0x344600) of every function in
# those ranges -> research/gfx/el_xref.txt
import idautils, idc
OUT = r"D:\projects\project poland rewrite\research\gfx\el_xref.txt"
R = [(0x2C5A60, 0x2D2E60), (0x317890, 0x3184A0), (0x343290, 0x344600)]
def inside(a):
    return any(s <= a < e for s, e in R)
out = []
for s, e in R:
    for fa in idautils.Functions(s, e):
        callers = set()
        for x in idautils.XrefsTo(fa, 0):
            if not inside(x.frm):
                f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
                callers.add("%X %s" % (x.frm, idc.get_func_name(f) if f != idc.BADADDR else "?"))
        if callers:
            out.append("== %X %s" % (fa, idc.get_func_name(fa)))
            for c in sorted(callers):
                out.append("   " + c)
open(OUT, "w").write("\n".join(out))
print(len(out))
