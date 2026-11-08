import idc, ida_hexrays
out = open(r"D:/projects/project poland rewrite/research/core_jumpcam.c", "w")
for f in [0x166490, 0x1665E0, 0x1665A0, 0x166B70]:
    out.write("\n// ===== %X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)))
    except Exception as e:
        out.write("// fail %s\n" % e)
out.close()
