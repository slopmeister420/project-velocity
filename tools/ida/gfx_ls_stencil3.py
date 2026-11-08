import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\ls_stencil3.c"
with open(OUT, "w", encoding="utf-8") as f:
    f.write("// refs dword_71FF10:")
    for x in idautils.DataRefsTo(0x71FF10):
        f.write(" %08X(%s)" % (x, idc.get_func_name(x)))
    f.write("\n")
    for fa in idautils.Functions(0x12D000, 0x131700):
        try:
            t = str(ida_hexrays.decompile(fa))
        except Exception:
            continue
        if "+ 484)" in t or "1936" in t or "Stencil" in t or "2A10C0" in t or "2A1140" in t:
            f.write("//==== %08X %s\n%s\n" % (fa, idc.get_func_name(fa), t))
print("ok")
