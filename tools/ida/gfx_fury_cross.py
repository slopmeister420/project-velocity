# decompile VsVector4::Cross and Normalise -> research/gfx/fury_cross.c
import ida_hexrays, idc, ida_name
out = "D:/projects/project poland rewrite/research/gfx/fury_cross.c"
with open(out, "w", encoding="utf-8") as f:
    for n in ["VsVector4::Cross", "VsVector4::Normalise", "VsTrafficManager__42DD0", "OSDManager__EFD30", "CHub__43FB0", "VertexStream__2B95D0"]:
        ea = ida_name.get_name_ea(idc.BADADDR, n)
        f.write("//==== %s %X\n" % (n, ea))
        try:
            f.write(str(ida_hexrays.decompile(ea)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
