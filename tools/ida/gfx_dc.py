# decompile functions at addresses listed in tools/ida/gfx_dc_args.txt: first line = output path, then one hex addr per line
import ida_hexrays, ida_funcs, idc, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_dc_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for a in lines[1:]:
        ea = int(a.split()[0], 16)
        fn = ida_funcs.get_func(ea)
        if not fn:
            f.write("// no func at %08X\n" % ea)
            continue
        f.write("//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
        try:
            f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
