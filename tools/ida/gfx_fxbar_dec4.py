import ida_hexrays, idc, struct
R = r"D:\projects\project poland rewrite\research\gfxdmg"
with open(R + r"\barrier_helpers2.c", "w", encoding="utf-8") as f:
    for a in (0x624A84, 0x624968):
        v = idc.get_wide_dword(a)
        f.write("// %08X %08X %r\n" % (a, v, struct.unpack('<f', v.to_bytes(4, 'little'))[0]))
    for fa in (0x825E0, 0x10B870, 0x2B99C0, 0x76620, 0x5E070, 0x12A7D0, 0x2AB250):
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
