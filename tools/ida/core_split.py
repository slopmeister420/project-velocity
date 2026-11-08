import ida_hexrays, ida_funcs, idc
out = []
for ea in (0x0005F0F0,):
    f = ida_funcs.get_func(ea)
    try:
        out.append("// %x %s\n" % (ea, idc.get_func_name(ea)) + str(ida_hexrays.decompile(f)))
    except Exception as e:
        out.append("// %x fail %s" % (ea, e))
open(r"D:\projects\project poland rewrite\research\core_split2.c", "w").write("\n".join(out))
