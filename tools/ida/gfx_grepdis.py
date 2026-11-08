# grep disassembly text: args file tools/ida/gfx_grepdis_args.txt = output path, then lines "<lo> <hi> <substring>"
import idautils, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_grepdis_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        lo, hi, s = a.split(None, 2)
        lo, hi = int(lo, 16), int(hi, 16)
        f.write("== %s in %X..%X\n" % (s, lo, hi))
        for h in idautils.Heads(lo, hi):
            d = idc.GetDisasm(h)
            if s in d:
                fn = ida_funcs.get_func(h)
                f.write("  %08X %s %s\n" % (h, idc.get_func_name(fn.start_ea) if fn else "?", d))
print("wrote", out)
