import ida_hexrays, idc, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\q3.c"
with open(OUT, "w", encoding="utf-8") as f:
    for a in [0x625C3C, 0x687620]:
        f.write("data %08X = %08X\n" % (a, ida_bytes.get_dword(a)))
    for fa in [0x13940, 0x825C0, 0x2B1370, 0x15E760, 0x1603F0]:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa))[:3000] + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
