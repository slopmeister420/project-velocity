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

dec_list(R + r"\barrier_helpers.c", [0x43FB0, 0x42DD0, 0x42E00, 0x2B8C70, 0x2B95D0, 0x2B90A0, 0x2A9A80, 0x825C0,
                                      0x763F0, 0x10B740, 0x10B7F0, 0x10B820, 0x10B840, 0x768D0, 0x2A15F0, 0x2A1760,
                                      0x4E6D0])
with open(R + r"\barrier_data.txt", "w") as f:
    def dump(a, n):
        for k in range(n):
            x = a + 4 * k
            v = idc.get_wide_dword(x)
            f.write("%08X %08X %d %r\n" % (x, v, v if v < 0x80000000 else v - (1 << 32), struct.unpack('<f', v.to_bytes(4, 'little'))[0]))
    dump(0x686560, 40)
    dump(0x705F40, 24)
    f.write("-- writes 6D63A0 / 705F40 / 705F84 / 705F98 / 705F88 / 6865B8\n")
    for a in (0x6D63A0, 0x705F40, 0x705F84, 0x705F98, 0x705F88, 0x6865B8, 0x686578, 0x68658C, 0x6F70E9):
        for x in idautils.XrefsTo(a):
            f.write("%08X <- %08X %s %s\n" % (a, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
    f.write("-- unk_63ABBC / off_63ABB8 / off_63AB78 strings\n")
    for a in (0x63ABBC, 0x63ABB8, 0x63AB78):
        v = idc.get_wide_dword(a)
        f.write("%08X %08X %r %r\n" % (a, v, idc.get_strlit_contents(a), idc.get_strlit_contents(v) if v > 0x10000 else None))
print("ok")
