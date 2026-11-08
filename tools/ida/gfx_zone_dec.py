import ida_hexrays, idc, idautils, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\zone_dec.c"
FN = [0x154BD0, 0x154FC0, 0x155070, 0x163740, 0x163770, 0x1637B0, 0x1637D0, 0x163800, 0x97E90, 0xBF060, 0xBFA50,
      0x15E780, 0x15E990, 0x15E7B0, 0x15E560, 0x15E650, 0x452C0, 0x57F90, 0x57F60, 0x3DDFA8,
      0xC62F0, 0xC62C0, 0xC6290, 0xC6330, 0x162B60]
DATA = [0x687668, 0x68766C, 0x687670, 0x687674, 0x687678, 0x748C00, 0x6868AC, 0x6868B0, 0x6868B4, 0x718AB8,
        0x6F5B70, 0x6865F0]
with open(OUT, "w", encoding="utf-8") as f:
    for a in DATA:
        f.write("// data %08X dword %08X float %r byte %d\n" % (a, ida_bytes.get_dword(a), idc.GetFloat(a) if hasattr(idc, "GetFloat") else 0, ida_bytes.get_byte(a)))
    for fa in FN:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    for t in [0x162D10, 0x1205E0, 0x120850, 0x11F860, 0x154BD0, 0x154FC0, 0x1202F0, 0x162B60]:
        f.write("// callers of %08X:" % t)
        for x in idautils.XrefsTo(t):
            f.write(" %08X(%s)" % (x.frm, idc.get_func_name(x.frm)))
        f.write("\n")
print("ok")
