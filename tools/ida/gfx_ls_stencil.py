import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\ls_stencil.txt"

def callers(f, fa, depth=0):
    for x in idautils.CodeRefsTo(fa, 0):
        fn = ida_funcs.get_func(x)
        f.write("%s%08X in %s\n" % ("  " * depth, x, idc.get_func_name(x)))
        if depth < 3 and fn and fn.start_ea != 0xB9010:
            callers(f, fn.start_ea, depth + 1)

with open(OUT, "w", encoding="utf-8") as f:
    f.write("//==== 12E5C0\n")
    try:
        f.write(str(ida_hexrays.decompile(0x12E5C0)) + "\n")
    except Exception as ex:
        f.write("// fail %s\n" % ex)
    for fa in [0x12EDC0, 0x137090, 0x14A5B0]:
        f.write("// callers of %08X %s\n" % (fa, idc.get_func_name(fa)))
        callers(f, fa)
print("ok")
