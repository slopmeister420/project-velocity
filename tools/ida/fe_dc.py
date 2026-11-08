# frontend agent: decompile functions named/addressed in fe_dc_args.txt -> research/fe_dc.c
import idc, ida_hexrays, ida_name
out = open(r"D:\projects\project poland rewrite\research\fe_dc.c", "w", encoding="utf-8")
for t in open(r"D:\projects\project poland rewrite\tools\ida\fe_dc_args.txt").read().split():
    try:
        ea = int(t, 16)
    except ValueError:
        ea = ida_name.get_name_ea(idc.BADADDR, t)
    f = idc.get_func_attr(ea, idc.FUNCATTR_START)
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
out.close()
