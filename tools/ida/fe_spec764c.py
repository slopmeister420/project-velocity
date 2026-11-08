# decompile FE3DCharacters (0x25F440..0x262500) and CElement::Init / 0x286180 to find where the char element's +764 is set
import idautils, idc, ida_hexrays
out = r"D:/projects/project poland rewrite/research/fe_spec764c.c"
with open(out, "w", encoding="utf-8") as f:
    eas = list(idautils.Functions(0x25F440, 0x262500)) + [0x285E90, 0x286180]
    for ea in eas:
        f.write("// ===== %08X %s\n" % (ea, idc.get_func_name(ea)))
        try:
            f.write(str(ida_hexrays.decompile(ea)) + "\n\n")
        except Exception as ex:
            f.write("// fail %s\n\n" % ex)
print("wrote", out)
