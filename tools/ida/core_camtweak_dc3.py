import ida_hexrays, idc
out = []
for ea in [0xC0BD0, 0xC0BF0, 0xB3950, 0x5F5D0, 0x602F0, 0x5C850, 0x13F70]:
    try:
        out.append("// ==== %X %s\n%s" % (ea, idc.get_func_name(ea), str(ida_hexrays.decompile(ea))))
    except Exception as e:
        out.append("// ==== %X fail %s" % (ea, e))
open(r"D:/projects/project poland rewrite/research/core_camtweak_dc3.c", "w").write("\n".join(out))
print("ok")
