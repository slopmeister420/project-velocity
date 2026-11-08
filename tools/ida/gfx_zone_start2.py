import ida_hexrays, idc, idautils, ida_funcs, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\zone_start2.txt"

with open(OUT, "w", encoding="utf-8") as f:
    f.write("6249C0 = %08X\n" % ida_bytes.get_dword(0x6249C0))
    for fa in [0x4B950, 0x4B9E0]:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    # string xrefs for heartbeat
    for s in idautils.Strings():
        t = str(s)
        if "HEARTBEAT" in t.upper() or "ZONE_" in t.upper():
            f.write("str %08X %s:" % (s.ea, t))
            for x in idautils.DataRefsTo(s.ea):
                f.write(" %08X(%s)" % (x, idc.get_func_name(x)))
            f.write("\n")
    # who writes +545 of unk_6F5BC8 / GameBridge pause; callers of 162D10 context
    for site in [0xB9E5C, 0xBA05B]:
        f.write("// site %08X\n" % site)
        a = site
        for _ in range(20):
            a = idc.prev_head(a)
        while a <= site:
            f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
            a = idc.next_head(a, site + 16)
print("ok")
