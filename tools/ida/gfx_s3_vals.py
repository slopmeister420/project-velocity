import idc, struct, ida_hexrays
out = open("D:/projects/project poland rewrite/research/gfx/s3_vals.txt", "w")
for a in range(0x686818, 0x686848, 4):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
for f in (0x452C0, 0x1C6480, 0x2AC070):
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n")
    except Exception as ex:
        out.write("// fail %X %s\n" % (f, ex))
out.close()
