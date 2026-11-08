# decompile functions by name listed in tools/ida/gfx_dc_names_args.txt: first line = output path, then one name per line
import ida_hexrays, ida_funcs, idc, idaapi, os
base = os.path.dirname(os.path.abspath(__file__))
lines = [l.strip() for l in open(os.path.join(base, "gfx_dc_names_args.txt")) if l.strip()]
out = lines[0]
with open(out, "w", encoding="utf-8") as f:
    for n in lines[1:]:
        ea = idc.get_name_ea_simple(n)
        fn = ida_funcs.get_func(ea) if ea != idaapi.BADADDR else None
        if not fn:
            f.write("// no func %s\n" % n)
            continue
        f.write("//==== %08X %s\n" % (fn.start_ea, n))
        try:
            f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
