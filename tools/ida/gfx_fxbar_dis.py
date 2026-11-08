import idc, struct
R = r"D:\projects\project poland rewrite\research\gfxdmg\damagefx_dis.txt"
with open(R, "w") as f:
    ea = 0x421E00
    while ea < 0x421F00:
        f.write("%08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea)
    for fa in (0x2AB540, 0x2AB560, 0x2AB580):
        f.write("== %08X\n" % fa)
        ea = fa
        for _ in range(12):
            f.write("%08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
            ea = idc.next_head(ea)
    for x in __import__('idautils').XrefsTo(0x76E80):
        f.write("76E80 <- %08X %s\n" % (x.frm, idc.get_func_name(x.frm)))
    for x in __import__('idautils').XrefsTo(0x70F40):
        f.write("70F40 <- %08X %s\n" % (x.frm, idc.get_func_name(x.frm)))
print("ok")
