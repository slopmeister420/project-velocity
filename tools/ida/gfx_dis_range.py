# disassemble ranges listed in tools/ida/gfx_dis_range_args.txt: first line = output path, then "lo hi" hex per line
import idautils, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_dis_range_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        lo, hi = [int(x, 16) for x in a.split()]
        f.write("== %08X..%08X\n" % (lo, hi))
        for h in idautils.Heads(lo, hi):
            f.write("%08X %s\n" % (h, idc.GetDisasm(h)))
print("wrote", out)
