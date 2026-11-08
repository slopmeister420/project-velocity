# ShaderManagerBase setters used by the puddle shaders -> research/gfx/pud6_setters.c
import ida_hexrays, idc
OUT = r"D:\projects\project poland rewrite\research\gfx\pud6_setters.c"
with open(OUT, "w", encoding="utf-8") as f:
    for a in (0x2A17D0, 0x2A1040, 0x2A15F0, 0x2A0F40, 0x2A0FC0):
        f.write("//==== %08X %s\n" % (a, idc.get_func_name(a)))
        try:
            f.write(str(ida_hexrays.decompile(a)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
