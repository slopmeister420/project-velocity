# bonnet smoke inputs: damage level 0x78090, car lookup 0x761B0, callers -> research/gfx/bs_lvl.c
import ida_hexrays, idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for ea in [0x766B0, 0x768D0, 0x76E80, 0x769E0, 0x78090]:
    out.append("== xrefs %X %s" % (ea, idc.get_func_name(ea)))
    for x in list(idautils.XrefsTo(ea))[:8]:
        f = ida_funcs.get_func(x.frm)
        out.append("  %X %s" % (x.frm, f and idc.get_func_name(f.start_ea)))
for a in [0x78090, 0x766B0, 0x768D0]:
    out.append("//==== %08X %s" % (a, idc.get_func_name(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
open(B + r"\bs_lvl.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
