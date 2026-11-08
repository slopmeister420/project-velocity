import ida_hexrays, idc
out = []
for ea in [0x5ECC0, 0x5F140, 0xB73D0, 0xB75D0, 0xB7730, 0xC0A90, 0x65610, 0xB88B0]:
    try:
        c = ida_hexrays.decompile(ea)
        out.append("// ==== %X %s\n%s" % (ea, idc.get_func_name(ea), str(c)))
    except Exception as e:
        out.append("// ==== %X fail %s" % (ea, e))
open(r"D:/projects/project poland rewrite/research/core_camtweak_dc.c", "w").write("\n".join(out))
print("ok")
