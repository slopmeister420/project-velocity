import ida_hexrays, idc
R = r"D:\projects\project poland rewrite\research\gfxdmg\particle_total.c"
with open(R, "w", encoding="utf-8") as f:
    for fa in (0x863B0,):
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
