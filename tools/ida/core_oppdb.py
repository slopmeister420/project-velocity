import idautils, idc, ida_hexrays, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\core_oppdb.c", "w")
for ea in idautils.Functions(0xE72C0, 0xE85A0):
    try:
        out.write("// ==== %X %s\n%s\n" % (ea, idc.get_func_name(ea), str(ida_hexrays.decompile(ea))))
    except Exception as e:
        out.write("// fail %X %s\n" % (ea, e))
out.close()
