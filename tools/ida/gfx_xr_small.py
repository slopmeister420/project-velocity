# xrefs to addrs in research/gfx/xr_in.txt -> research/gfx/xr_out.txt (caller func + site)
import idautils, idc, ida_funcs
base = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(base + r"\xr_in.txt"):
    l = l.strip()
    if not l: continue
    a = int(l, 16)
    out.append("== %X %s" % (a, idc.get_func_name(a)))
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        out.append("  %X in %s (%X)" % (x.frm, idc.get_func_name(x.frm), f.start_ea if f else 0))
open(base + r"\xr_out.txt", "w").write("\n".join(out) + "\n")
