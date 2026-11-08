import ida_hexrays, idc, ida_name
out = open("D:/projects/project poland rewrite/research/gfx/s3_sm.c", "w", encoding="utf-8")
cs = ida_name.get_name_ea(idc.BADADDR, "CShaderManagerBase::CreateShader")
for f in (cs, 0x2A0DD0, 0x2A0F40, 0x2A0FC0, 0x2A1040, 0x2A0AA0, 0x15E740):
    try:
        out.write("//==== %X\n%s\n" % (f, ida_hexrays.decompile(f)))
    except Exception as ex:
        out.write("// fail %X %s\n" % (f, ex))
out.write("6870F8 %02X\n" % idc.get_wide_byte(0x6870F8))
out.close()
print("ok")
