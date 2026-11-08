import ida_hexrays, idc, idautils, struct
R = r"D:\projects\project poland rewrite\research\gfxdmg"

def dec_list(out, fns):
    with open(out, "w", encoding="utf-8") as f:
        for fa in fns:
            f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
            try:
                f.write(str(ida_hexrays.decompile(fa)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)

dec_list(R + r"\damagefx_helpers2.c", [0x76E80, 0x70F40, 0x2AB7E0, 0x2D5920, 0x2D57D0, 0x2B6E90, 0x7A840, 0x2AC0A0,
                                        idc.get_func_attr(0x421E6F, idc.FUNCATTR_START)])
with open(R + r"\damagefx_data.txt", "w") as f:
    for a in range(0x6D69B8, 0x6D69B8 + 64, 4):
        v = idc.get_wide_dword(a)
        f.write("%08X %08X %r\n" % (a, v, struct.unpack('<f', v.to_bytes(4, 'little'))[0]))
    # disasm of the 78880 call sites for 2AB330 args
    ea = 0x78880
    end = idc.get_func_attr(ea, idc.FUNCATTR_END)
    while ea < end:
        d = idc.generate_disasm_line(ea, 0)
        if "2AB330" in d or "2ABDA0" in d or "2AB560" in d or "2AC070" in d:
            p = ea
            for _ in range(5):
                p = idc.prev_head(p)
            q = p
            while q <= ea:
                f.write("%08X %s\n" % (q, idc.generate_disasm_line(q, 0)))
                q = idc.next_head(q)
            f.write("--\n")
        ea = idc.next_head(ea)
print("ok")
