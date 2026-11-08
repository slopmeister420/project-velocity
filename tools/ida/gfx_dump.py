# decompile every function in the ranges listed in tools/ida/gfx_ranges.txt ("start end out" per line)
import idautils, ida_hexrays, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
for line in open(os.path.join(base, "gfx_ranges.txt")):
    p = line.split()
    if len(p) < 3:
        continue
    s, e, out = int(p[0], 16), int(p[1], 16), " ".join(p[2:])
    with open(out, "w", encoding="utf-8") as f:
        for fa in idautils.Functions(s, e):
            f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
            try:
                f.write(str(ida_hexrays.decompile(fa)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)
    print("wrote", out)
