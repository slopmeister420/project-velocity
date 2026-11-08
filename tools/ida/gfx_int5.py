# who sets the D3D material (fixed-function car prims' DIFFUSE): callers of *SetMaterial* -> research/gfx/int5.txt
import idc, idautils, ida_funcs, ida_name
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
targets = []
for ea, name in idautils.Names():
    n = name.lower()
    if "setmaterial" in n or "materialdiffuse" in n or "colorvertex" in n or "diffusematerialsource" in n:
        targets.append((ea, name))
for ea, name in targets:
    out.append("== %X %s" % (ea, name))
    for x in idautils.XrefsTo(ea):
        out.append("  %X in %s" % (x.frm, idc.get_func_name(x.frm)))
open(B + r"\int5.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(targets))
