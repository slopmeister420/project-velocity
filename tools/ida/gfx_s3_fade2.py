import ida_hexrays, idc, ida_funcs, struct
out = open("D:/projects/project poland rewrite/research/gfx/s3_fade3.c", "w", encoding="utf-8")
for f in (0x122FB0, 0x123030, 0x123070, 0x1230C0, 0xB3710):
    try:
        out.write("//==== %X\n%s\n" % (f, ida_hexrays.decompile(f)))
    except Exception as ex:
        out.write("// fail %X %s\n" % (f, ex))
for a in (0xB826C, 0xBDEF8, 0xBDF10, 0xBDF31, 0x4250B3):
    out.write("// ctx %X\n" % a)
    ea = a
    for _ in range(6):
        ea = idc.prev_head(ea)
    for _ in range(14):
        out.write("   %X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
for a in (0x6869A0, 0x6869C8):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
out.close()
print("ok")
