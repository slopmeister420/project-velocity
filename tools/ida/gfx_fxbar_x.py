import idc, idautils, ida_hexrays
R = r"D:\projects\project poland rewrite\research\gfxdmg\damagefx_x.txt"
with open(R, "w") as f:
    for a in (0x768D0, 0x109080):
        for x in idautils.XrefsTo(a):
            f.write("%08X <- %08X %s\n" % (a, x.frm, idc.get_func_name(x.frm)))
            p = x.frm
            for _ in range(8):
                p = idc.prev_head(p)
                f.write("   %08X %s\n" % (p, idc.generate_disasm_line(p, 0)))
    f.write(str(ida_hexrays.decompile(idc.get_name_ea_simple("VsVector4::VsVector4_XYZ"))) + "\n")
print("ok")
