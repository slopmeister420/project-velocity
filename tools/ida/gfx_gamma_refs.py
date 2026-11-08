# who uses SetGammaRamp and the "Brightness ( Multiply - free)" string -> research/gfx/gamma_refs.txt
import idautils, idc, ida_funcs, idaapi
out = []
for ea, name in idautils.Names():
    if "GammaRamp" in name or "gamma" in name.lower():
        refs = ["%X %s" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(ea, 0)]
        out.append("name %X %s <- %s" % (ea, name, refs[:20]))
for s in (0x62f7fc, 0x63b1f4, 0x63bccc):
    refs = ["%X %s" % (x, idc.get_func_name(x)) for x in idautils.DataRefsTo(s)]
    out.append("str %X %s <- %s" % (s, idc.get_strlit_contents(s), refs[:10]))
open(r"D:/projects/project poland rewrite/research/gfx/gamma_refs.txt", "w").write("\n".join(out) + "\n")
