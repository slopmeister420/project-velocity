import ida_hexrays, idautils, idc
out = []
for ea in (0x29A120, 0x29A220, 0x29A310):
    out.append("// ===== %08X\n" % ea + str(ida_hexrays.decompile(ea)))
    out.append("// callers: " + " ".join("%X:%s" % (x, idc.get_func_name(x)) for x in idautils.CodeRefsTo(ea, 0)))
open(r"D:/projects/project poland rewrite/research/fe_cam_add.c", "w").write("\n".join(out))
