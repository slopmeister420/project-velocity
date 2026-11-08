# decompile every function starting in the ranges of tools/ida/gfx_dc_range_args.txt: first line = output path, then "lo hi" hex
import ida_hexrays, idautils, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_dc_range_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        lo, hi = [int(x, 16) for x in a.split()]
        for ea in idautils.Functions(lo, hi):
            f.write("//==== %08X %s\n" % (ea, idc.get_func_name(ea)))
            try:
                f.write(str(ida_hexrays.decompile(ea)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)
print("wrote", out)
