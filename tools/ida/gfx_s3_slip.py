import ida_hexrays, idc, ida_funcs, idautils, struct
out = open("D:/projects/project poland rewrite/research/gfx/s3_slip3.c", "w", encoding="utf-8")
for f in (0x15EC80, 0x15E9E0, 0x130070, 0x13F380, 0x57200, 0xC8CF0, 0x12D490, 0x248C0, 0x1D660):
    try:
        out.write("//==== %X\n%s\n" % (f, ida_hexrays.decompile(f)))
    except Exception as ex:
        out.write("// fail %X %s\n" % (f, ex))
for a in (0x687130, 0x687134, 0x687138, 0x68713C, 0x687140, 0x686BD4, 0x686BD8):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
for t in (0x1629B0, 0x15FB30, 0x123250):
    for x in idautils.XrefsTo(t):
        fn = ida_funcs.get_func(x.frm)
        out.write("// %X xref %X in %s\n" % (t, x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
        ea = x.frm
        for _ in range(14):
            ea = idc.prev_head(ea)
        for _ in range(18):
            out.write("   %X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
            ea = idc.next_head(ea)
out.close()
print("ok")
