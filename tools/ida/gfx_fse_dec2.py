import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\fse_dec2.c"
FN = [0x162FF0,0x163740,0x154BD0,0x154FC0,0x155070,0x128680,0x1E0EE0]
with open(OUT, "w", encoding="utf-8") as f:
    for fa in FN:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    for ea in [0x154BD0, 0x154FC0, 0x162D10, 0x686C20, 0x71FF38]:
        f.write("// xrefs %x\n" % ea)
        for x in idautils.XrefsTo(ea):
            fn = ida_funcs.get_func(x.frm)
            f.write("//   %x %s\n" % (x.frm, fn and idc.get_func_name(fn.start_ea)))
print("ok")
