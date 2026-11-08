# find names matching the substrings in tools/ida/gfx_names_args.txt (first line = output path), with callers
import idautils, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_names_args.txt")) if l.strip()]
out = lines[0]
pats = [p.lower() for p in lines[1:]]
with open(out, "w", encoding="utf-8") as f:
    for ea, name in idautils.Names():
        if any(p in name.lower() for p in pats):
            f.write("== %08X %s\n" % (ea, name))
            seen = set()
            for x in idautils.XrefsTo(ea):
                fn = ida_funcs.get_func(x.frm)
                k = fn.start_ea if fn else x.frm
                if k in seen:
                    continue
                seen.add(k)
                f.write("  %08X %s\n" % (x.frm, idc.get_func_name(k) if fn else "?"))
print("wrote", out)
