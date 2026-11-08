# xrefs to addresses listed in tools/ida/gfx_xr_args.txt: first line = output path, then one hex addr per line
import idautils, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_xr_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        ea = int(a.split()[0], 16)
        f.write("== %08X %s\n" % (ea, idc.get_name(ea)))
        for x in idautils.XrefsTo(ea):
            fn = ida_funcs.get_func(x.frm)
            f.write("  %08X %s %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?", idc.GetDisasm(x.frm)))
print("wrote", out)
