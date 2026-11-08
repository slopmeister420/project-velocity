# frontend agent: code xrefs to addresses in fe_xr_args.txt -> research/fe_xr.txt
import idc, idautils
base = r"D:\projects\project poland rewrite"
with open(base + r"\research\fe_xr.txt", "w") as o:
    for t in open(base + r"\tools\ida\fe_xr_args.txt").read().split():
        ea = int(t, 16)
        o.write("== %08X %s\n" % (ea, idc.get_func_name(ea)))
        for x in idautils.XrefsTo(ea):
            f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
            o.write("  %08X in %08X %s\n" % (x.frm, f if f != idc.BADADDR else 0, idc.get_func_name(x.frm)))
print("ok")
