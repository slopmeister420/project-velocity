import ida_hexrays, ida_funcs
addrs = [0x2528D0, 0x2515D0, 0x250EE0, 0xC8190, 0xC9C10, 0xF3A70, 0x2EC810, 0x207440, 0x207730, 0x281470, 0x293810, 0x293E40, 0x276810, 0x275F10, 0x274B40, 0x274EE0, 0x272AF0, 0x25E250, 0xB3170, 0x2999B0, 0x2F6040, 0x2F3160]
out = open(r"D:/projects/project poland rewrite/research/fe_tri_dec.c", "w")
for a in addrs:
    out.write("\n//==== 0x%X\n" % a)
    try:
        out.write(str(ida_hexrays.decompile(a)))
    except Exception as e:
        out.write("ERR %s\n" % e)
out.close()
