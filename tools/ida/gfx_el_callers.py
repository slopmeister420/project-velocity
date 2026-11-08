# callers (code + data refs) of the addresses in research/gfx/el_callers_in.txt -> el_callers_out.txt
import idautils, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_callers_in.txt"):
    l = l.split("#")[0].strip()
    if not l:
        continue
    a = int(l, 16)
    out.append("== %X %s" % (a, idc.get_name(a)))
    for x in idautils.XrefsTo(a, 0):
        fn = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.append("   %X %s" % (x.frm, idc.get_func_name(fn) if fn != idc.BADADDR else "?"))
open(B + r"\el_callers_out.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
