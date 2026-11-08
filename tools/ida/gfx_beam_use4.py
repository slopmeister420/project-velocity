# beam lights: how BindShader uploads matrix inputs (0x2DF7F0), and the world matrix resolver
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_use4.c"
with open(out, "w", encoding="utf-8") as f:
    for t in (0x2DF7F0, 0x12C1A0):
        fn = ida_funcs.get_func(t)
        if fn is None:
            f.write("// no func %08X\n" % t)
            continue
        f.write("\n//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
        try:
            f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
