import ida_hexrays, idautils, idc, ida_funcs
RANGES = [
    ("profile", 0x00232C60, 0x00234EB0),
    ("hangout", 0x00219F70, 0x0021FA60),
    ("progression", 0x0028F430, 0x00292FF0),
]
base = r"D:\projects\project poland rewrite\research\fe_career_%s.c"
idx = open(r"D:\projects\project poland rewrite\research\fe_career_index.txt", "w")
for tag, s, e in RANGES:
    out = open(base % tag, "w")
    for f in idautils.Functions(s, e):
        fn = ida_funcs.get_func(f)
        name = idc.get_func_name(f)
        idx.write("%s %08x %5d %s\n" % (tag, f, fn.end_ea - fn.start_ea, name))
        try:
            txt = str(ida_hexrays.decompile(f))
        except Exception as ex:
            txt = "// decompile failed %s" % ex
        out.write("// ==== %08x %s\n%s\n\n" % (f, name, txt))
    out.close()
idx.close()
