# beam lights, next-stage groundwork: callers of the ProjectedLightBeam entry points and decompiles of the
# texture / transform builders (GameShaderProgramManager-range 0x161190 / 0x161520 / 0x1618C0 / 0x1618D0)
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_dc2.c"
with open(out, "w", encoding="utf-8") as f:
    for t in (0x8ABB0, 0x161C40, 0x161F10, 0x161E50, 0x1618D0, 0x1618C0, 0x161190, 0x161520, 0xA4CF0):
        for x in idautils.XrefsTo(t):
            fn = ida_funcs.get_func(x.frm)
            f.write("// xref %08X -> %08X in %s\n" % (x.frm, t, idc.get_func_name(fn.start_ea) if fn else "?"))
    for t in (0x161190, 0x161520, 0x1618C0, 0x1618D0, 0xA4CF0):
        f.write("\n//==== %08X %s\n" % (t, idc.get_func_name(t)))
        try:
            f.write(str(ida_hexrays.decompile(t)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
