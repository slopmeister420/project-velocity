# wheel blur -> research/gfx/wblur_dec.c (0x116F60, its callers, "wblur" string refs)
import ida_hexrays, idc, idautils, ida_bytes, idaapi
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
seen = set()
def dec(a):
    f = idaapi.get_func(a)
    if not f or f.start_ea in seen:
        return
    seen.add(f.start_ea)
    out.append("//==== %08X %s  [%s]" % (f.start_ea, idc.get_func_name(f.start_ea), idc.get_type(f.start_ea)))
    try:
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as ex:
        out.append("// fail %s" % ex)
    out.append("// callers: " + " ".join("%X(%s)" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(f.start_ea, 0)))
dec(0x116F60)
for x in idautils.CodeRefsTo(0x116F60, 0):
    out.append("// caller %X" % x)
# strings
import re
for line in open(r"D:\projects\project poland rewrite\research\strings.tsv", encoding="utf-8", errors="replace"):
    p = line.rstrip("\n").split("\t", 1)
    if len(p) == 2 and re.search(r"wblur|blur_0|wheel_spokes|spokes", p[1], re.I):
        a = int(p[0], 16)
        out.append("// str %X %s" % (a, p[1]))
        for x in idautils.DataRefsTo(a):
            out.append("//   ref %X in %s" % (x, idc.get_func_name(x)))
open(B + r"\wblur_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
