# beam lights port: object address, texture format, swizzle, compartment override caller, resolver, d3dx names
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_port.c"
with open(out, "w", encoding="utf-8") as f:
    for lo, hi in ((0xB62A0, 0xB62D0), (0xB9020, 0xB9060)):
        f.write("\n// dis %08X..%08X\n" % (lo, hi))
        ea = lo
        while ea < hi:
            f.write("%08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
            ea = idc.next_head(ea, hi + 16)
    for t in (0x48A0F9, 0x48858F, 0x488600, 0x489A80, 0x48815B, 0x3DDFA8, 0x445290, 0x4452D0):
        f.write("// name %08X %s\n" % (t, idc.get_func_name(t)))
    for t in (0x29EFF0, 0x1610C0, 0x8AC20, 0x161050, 0x161090, 0x161B70, 0x761E0):
        f.write("\n//==== %08X %s\n" % (t, idc.get_func_name(t)))
        try:
            f.write(str(ida_hexrays.decompile(t)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
