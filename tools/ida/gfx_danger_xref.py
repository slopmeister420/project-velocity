import ida_hexrays, idc, idautils, ida_bytes, ida_funcs, ida_segment
OUT = r"D:\projects\project poland rewrite\research\gfx\danger_xref.txt"
seen = set()
with open(OUT, "w", encoding="utf-8") as f:
    for a in range(0x718AB0, 0x718BA8):
        for x in idautils.DataRefsTo(a):
            fn = ida_funcs.get_func(x)
            f.write("%08X <- %08X %s  %s\n" % (a, x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
            if fn:
                seen.add(fn.start_ea)
    for a in [0x737AB0, 0x737AD8, 0x6D55C8]:
        for x in idautils.DataRefsTo(a):
            f.write("%08X <- %08X %s  %s\n" % (a, x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
    s = ida_segment.getseg(0x718AB8)
    f.write("seg %s %08X..%08X\n" % (ida_segment.get_segm_name(s), s.start_ea, s.end_ea))
    f.write("is_loaded 718AB8=%d 748C00=%d\n" % (ida_bytes.is_loaded(0x718AB8), ida_bytes.is_loaded(0x748C00)))
    for fa in sorted(seen):
        if 0x11F3A0 <= fa < 0x121500:
            continue
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            t = str(ida_hexrays.decompile(fa))
            f.write(t[:6000] + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
