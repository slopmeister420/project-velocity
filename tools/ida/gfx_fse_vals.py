import ida_bytes, struct, ida_hexrays, idc
OUT = r"D:\projects\project poland rewrite\research\gfx\fse_vals.txt"
def f32(ea):
    return struct.unpack("<f", struct.pack("<I", ida_bytes.get_dword(ea)))[0]
with open(OUT, "w") as o:
    for ea in list(range(0x686C20, 0x686C3C, 4)) + [0x6870D4, 0x6870D8, 0x77FD9C, 0x6F7157, 0x6875D0]:
        o.write("%x %08x %r b=%d\n" % (ea, ida_bytes.get_dword(ea), f32(ea), ida_bytes.get_byte(ea)))
    for fa in [0x24870, 0x133970, 0x1C66C0, 0x1BA270, 0x130070]:
        o.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            o.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            o.write("// fail %s\n" % ex)
print("ok")
