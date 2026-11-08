# beam lights: global input getters, SaveFileData shaders 4/5, sampler state 15D070
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_use2.c"
with open(out, "w", encoding="utf-8") as f:
    for t in (0x12D320, 0x12D340, 0x15D070, 0x106A40):
        f.write("\n//==== %08X %s\n" % (t, idc.get_func_name(t)))
        try:
            f.write(str(ida_hexrays.decompile(t)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    for t in (0x12C8F0, 0x12C910):
        for x in idautils.XrefsTo(t):
            f.write("// xref %08X -> %08X in %s : %s\n" % (x.frm, t, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
    f.write("\n//==== 001065A0 %s\n" % idc.get_func_name(0x1065A0))
    try:
        f.write(str(ida_hexrays.decompile(0x1065A0)) + "\n")
    except Exception as ex:
        f.write("// fail %s\n" % ex)
print("wrote", out)
