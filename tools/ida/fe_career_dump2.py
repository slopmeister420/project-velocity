import ida_hexrays, idautils, idc, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\fe_career_hangout0.c", "w")
idx = open(r"D:\projects\project poland rewrite\research\fe_career_index.txt", "a")
for f in idautils.Functions(0x002181A0, 0x00219F70):
    fn = ida_funcs.get_func(f)
    name = idc.get_func_name(f)
    idx.write("hangout0 %08x %5d %s\n" % (f, fn.end_ea - fn.start_ea, name))
    try:
        txt = str(ida_hexrays.decompile(f))
    except Exception as ex:
        txt = "// decompile failed %s" % ex
    out.write("// ==== %08x %s\n%s\n\n" % (f, name, txt))
out.close()
idx.close()
