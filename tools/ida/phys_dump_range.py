# physics agent: decompile every function in [start,end) listed in research/phys_range_args.txt (lines: start end outfile)
import idautils, ida_hexrays, ida_name
for line in open(r"D:\projects\project poland rewrite\research\phys_range_args.txt"):
    p = line.split(None, 2)
    if len(p) < 3:
        continue
    s, e, out = int(p[0], 16), int(p[1], 16), p[2].strip()
    n = 0
    with open(out, "w", encoding="utf-8") as o:
        for f in idautils.Functions(s, e):
            n += 1
            o.write("\n// ==== %08X %s\n" % (f, ida_name.get_name(f)))
            try:
                o.write(str(ida_hexrays.decompile(f)))
            except Exception as ex:
                o.write("// decompile failed: %s\n" % ex)
    print(out, n)
