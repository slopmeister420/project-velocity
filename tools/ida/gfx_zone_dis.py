import ida_hexrays, idc, idautils, ida_bytes, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\zone_dis.txt"

def dis(f, s, e):
    a = s
    while a < e:
        f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
        a = idc.next_head(a, e + 16)

with open(OUT, "w", encoding="utf-8") as f:
    for site in [0xB950D, 0xB9E5C, 0xB9E77, 0xBA05B]:
        f.write("// site %08X\n" % site)
        a = site
        for _ in range(12):
            a = idc.prev_head(a)
        dis(f, a, site + 6)
    fn = ida_funcs.get_func(0x162D10)
    f.write("// 162D10 head\n")
    dis(f, 0x162D10, 0x162D10 + 0x90)
    f.write("// 120A60\n")
    dis(f, 0x120A60, 0x120AC0)
    f.write("// 11F860 frame time part\n")
    dis(f, 0x11FA20, 0x11FB40)
    f.write("// table 6874BC..687540\n")
    a = 0x6874BC
    while a < 0x687540:
        f.write("%08X %08X %r\n" % (a, ida_bytes.get_dword(a), idc.get_float(a) if hasattr(idc, "get_float") else 0))
        a += 4
    for a in [0x687610, 0x687524]:
        f.write("%08X %08X\n" % (a, ida_bytes.get_dword(a)))
    for fa in [0x15EA60, 0x154F00, 0xC0450, 0x155990, 0x23310]:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
