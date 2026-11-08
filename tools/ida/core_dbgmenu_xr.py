import idautils, idc, ida_funcs
# code + data refs (with 4 preceding instructions) to the addresses in research/core_dbgmenu_xr_in.txt
# -> research/core_dbgmenu_xr.txt
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_dbgmenu_xr.txt", "w")
for line in open(base + r"\core_dbgmenu_xr_in.txt"):
    s = line.split()
    if not s:
        continue
    t = int(s[0], 16)
    out.write("== %x %s\n" % (t, idc.get_name(t)))
    refs = set(idautils.CodeRefsTo(t, 0)) | set(idautils.DataRefsTo(t))
    for x in sorted(refs):
        out.write("  %x in %s\n" % (x, idc.get_func_name(x)))
        p = x
        ctx = []
        for _ in range(4):
            p = idc.prev_head(p)
            ctx.append("      %x %s" % (p, idc.GetDisasm(p)))
        out.write("\n".join(reversed(ctx)) + "\n")
        out.write("    > %s\n" % idc.GetDisasm(x))
out.close()
print("ok")
