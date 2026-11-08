# names containing "cull" (any case) and their code xrefs from game code
import idautils, idc, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/cull_calls2.txt"
with open(out, "w", encoding="utf-8") as f:
    for ea, name in idautils.Names():
        if "cull" not in name.lower():
            continue
        refs = []
        for x in idautils.XrefsTo(ea, 0):
            fn = ida_funcs.get_func(x.frm)
            refs.append("%08X(%s)" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
        f.write("%08X %s : %s\n" % (ea, name, " ".join(refs[:30])))
print("wrote", out)
