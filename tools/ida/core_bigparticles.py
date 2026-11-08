import idautils, idc, ida_funcs
out = []
for t in (0x14DF70, 0x14E2A0, 0x14E280, 0x14E530):
    for x in idautils.XrefsTo(t):
        f = ida_funcs.get_func(x.frm)
        out.append("to %x from %x func %x" % (t, x.frm, f.start_ea if f else 0))
        ea = x.frm
        for i in range(12):
            ea = idc.prev_head(ea)
            d = idc.GetDisasm(ea)
            if "offset" in d or "push" in d:
                out.append("   %x %s" % (ea, d))
open(r"D:\projects\project poland rewrite\research\core_tbrender_xrefs.txt", "w").write("\n".join(out))
