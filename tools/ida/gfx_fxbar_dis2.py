import idc
R = r"D:\projects\project poland rewrite\research\gfxdmg\barrier_dis.txt"
with open(R, "w") as f:
    for a, b in ((0x424400, 0x4244E0), (0x4218C0, 0x421900), (0x6B8B0, 0x6B930)):
        ea = a
        while ea < b:
            f.write("%08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
            ea = idc.next_head(ea)
        f.write("--\n")
print("ok")
