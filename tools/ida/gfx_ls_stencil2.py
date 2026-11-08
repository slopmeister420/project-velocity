import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\ls_stencil2.c"
with open(OUT, "w", encoding="utf-8") as f:
    for fa in [0x12E770, 0x12EDC0, 0x12F250]:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    for site in [0xB9B71, 0xB9CE8, 0xB9D04]:
        f.write("// site %08X\n" % site)
        a = site
        for _ in range(14):
            a = idc.prev_head(a)
        while a <= site:
            f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
            a = idc.next_head(a, site + 16)
print("ok")
