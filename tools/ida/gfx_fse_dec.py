import ida_hexrays, idc
OUT = r"D:\projects\project poland rewrite\research\gfx\fse_dec.c"
FN = [0x2A10C0,0x2A1140,0x2A11C0,0x2A0F40,0x2A0FC0,0x2A1500,0x2A1340,0x2A1240,0x2A0DD0,0x2A1BE0,0x2A15F0,
      0x15AE50,0x15AF70,0x15C140,0x1629B0,0x162B60,0x162D10,0x13FDB0,0x2EEA10,0x2EF0B0,0x2EF030,0x2EEFE0,0x2EEE20]
with open(OUT, "w", encoding="utf-8") as f:
    for fa in FN:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
