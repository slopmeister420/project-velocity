# frontend agent: callers of FE3DCharacters show/hide and of the hangout Start/Leave -> research/fe_scene_xr.txt
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_scene_xr.txt", "w")
for ea in (0x219300, 0x219920, 0x261510, 0x2615F0, 0x2616A0):
    out.write("== %08X %s\n" % (ea, idc.get_func_name(ea)))
    for x in idautils.XrefsTo(ea, 0):
        out.write("  %08X %s type %d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
out.close()
