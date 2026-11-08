# beam lights: lighting inputs for programs (GFXHelper 12C670 / 12C770), sampler state 15AE80, 2A15F0
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_use3.c"
with open(out, "w", encoding="utf-8") as f:
    for t in (0x12C670, 0x12C770, 0x15AE80, 0x2A15F0):
        f.write("\n//==== %08X %s\n" % (t, idc.get_func_name(t)))
        try:
            f.write(str(ida_hexrays.decompile(t)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
