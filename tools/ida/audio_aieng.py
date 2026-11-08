import idautils, idc, ida_funcs, ida_hexrays
# ai engine audio: LoadAiEngine 0x531D0 + UpdateAiCars 0x53290 decomp, callers of LoadAiEngine
OUT = r"D:\projects\project poland rewrite\research\audio\ai_engine.c"
with open(OUT, "w", encoding="utf-8") as fo:
    for x in idautils.CodeRefsTo(0x531D0, 0):
        g = ida_funcs.get_func(x)
        fo.write("// caller %x in %s\n" % (x, idc.get_func_name(g.start_ea) if g else "?"))
    for ea in (0x531D0, 0x53290):
        try:
            c = str(ida_hexrays.decompile(ea))
        except Exception as e:
            c = "// decompile failed: %s" % e
        fo.write("// ===== %x %s\n%s\n\n" % (ea, idc.get_func_name(ea), c))
print("ok")
