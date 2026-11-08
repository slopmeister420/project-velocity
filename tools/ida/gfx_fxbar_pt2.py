import idc, idautils
R = r"D:\projects\project poland rewrite\research\gfxdmg\particle_round.txt"
with open(R, "a") as f:
    for a in (0x322680, 0x3226B0):
        for x in idautils.XrefsTo(a):
            f.write("%08X <- %08X %s\n" % (a, x.frm, idc.get_func_name(x.frm)))
print("ok")
