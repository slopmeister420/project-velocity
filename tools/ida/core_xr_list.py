import idc, idautils, ida_funcs
# callers (code + data refs) of the addresses in research/core_xr_in.txt -> research/core_xr_out.txt
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_xr_out.txt", "w")
for line in open(base + r"\core_xr_in.txt"):
    s = line.strip()
    if not s:
        continue
    ea = int(s, 16)
    refs = set()
    for x in list(idautils.CodeRefsTo(ea, 0)) + list(idautils.DataRefsTo(ea)):
        g = ida_funcs.get_func(x)
        refs.add("%x(%s @%x)" % (g.start_ea if g else 0, idc.get_func_name(x) if g else "?", x))
    out.write("%x %s <- %s\n" % (ea, idc.get_func_name(ea), ", ".join(sorted(refs))))
out.close()
print("ok")
