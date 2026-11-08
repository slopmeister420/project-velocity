# frontend agent: for each address in fe_gap_xr_args.txt: callers / data refs and the strings the function references
import idautils, idc, ida_bytes
out = open(r"D:\projects\project poland rewrite\research\fe_gap_xr.txt", "w", encoding="utf-8")
for t in open(r"D:\projects\project poland rewrite\tools\ida\fe_gap_xr_args.txt").read().split():
    ea = int(t, 16)
    f = idc.get_func_attr(ea, idc.FUNCATTR_START)
    out.write("== %08X %s size %d\n" % (f, idc.get_func_name(f), idc.get_func_attr(f, idc.FUNCATTR_END) - f))
    for x in idautils.XrefsTo(f, 0):
        out.write("  ref %08X %s type %d\n" % (x.frm, idc.get_func_name(x.frm) or "(data)", x.type))
        if not idc.get_func_name(x.frm):
            for y in idautils.XrefsTo(x.frm & ~3, 0):
                out.write("    dref %08X %s\n" % (y.frm, idc.get_func_name(y.frm)))
    ss = []
    for h in idautils.FuncItems(f):
        for d in idautils.DataRefsFrom(h):
            s = idc.get_strlit_contents(d, -1, 0)
            if s and len(s) > 2:
                ss.append(s.decode("latin1"))
    out.write("  strings: %s\n" % " | ".join(ss[:30]))
out.close()
