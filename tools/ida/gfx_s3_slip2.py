import ida_hexrays, idc, ida_funcs, idautils, struct
out = open("D:/projects/project poland rewrite/research/gfx/s3_slip4.txt", "w", encoding="utf-8")
ea = 0x140700
while ea < 0x140820:
    out.write("   %X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea)
# find the function start: walk back to a 'push ebp' preceded by align/ret
for t in (0x140780, 0x140790, 0x1407A0, 0x1407B0, 0x140770, 0x140760):
    for x in idautils.XrefsTo(t):
        fn = ida_funcs.get_func(x.frm)
        out.write("// xref to %X from %X in %s\n" % (t, x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
for a in (0x6870E0, 0x687614, 0x687620):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
for x in idautils.XrefsTo(0x6870E0):
    fn = ida_funcs.get_func(x.frm)
    out.write("// 6870E0 xref %X in %s: %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?", idc.generate_disasm_line(x.frm, 0)))
for x in idautils.XrefsTo(0x13FDB0):
    fn = ida_funcs.get_func(x.frm)
    out.write("// 13FDB0 xref %X in %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
out.close()
print("ok")
