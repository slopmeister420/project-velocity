import ida_hexrays, idc, ida_funcs, idautils, struct
out = open("D:/projects/project poland rewrite/research/gfx/s3_lens3.c", "w", encoding="utf-8")
for f in (0x12D400, 0x12D960, 0x133910, 0x12DC20, 0x57F90, 0x24870, 0x12D3E0):
    try:
        out.write("//==== %X\n%s\n" % (f, ida_hexrays.decompile(f)))
    except Exception as ex:
        out.write("// fail %X %s\n" % (f, ex))
for a in (0x686B94, 0x686B98, 0x686B9C, 0x686BA0, 0x686BA4, 0x686BC4, 0x686BC8, 0x686BCC, 0x686BD0, 0x725E20):
    out.write("%X %r %08X\n" % (a, struct.unpack('<f', idc.get_bytes(a, 4))[0], idc.get_wide_dword(a)))
for x in idautils.XrefsTo(0x725E20):
    fn = ida_funcs.get_func(x.frm)
    out.write("// 725E20 xref %X in %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
for x in idautils.XrefsTo(0x133910):
    fn = ida_funcs.get_func(x.frm)
    out.write("// 133910 xref %X in %s\n" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?"))
out.close()
print("ok")
