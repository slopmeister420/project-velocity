import idc, ida_hexrays, idautils
OUT = r"D:\projects\project poland rewrite\research\gfx\tb_dis.txt"

def dis(f, s, e):
    a = s
    while a < e:
        f.write("%08X  %s\n" % (a, idc.generate_disasm_line(a, 0)))
        a = idc.next_head(a, e + 16)

with open(OUT, "w", encoding="utf-8") as f:
    dis(f, 0xC5F90, 0xC6080)
    f.write("//---- setters\n")
    for fa in [0xC63A0, 0xC6380, 0xC6270, 0xC6360, 0xC63C0, 0xC63E0]:
        f.write("//==== %08X %s\n" % (fa, idc.get_func_name(fa)))
        try:
            f.write(str(ida_hexrays.decompile(fa)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
    for t in [0xC5F90, 0xC6080, 0xC6120]:
        f.write("// callers of %08X:" % t)
        for x in idautils.XrefsTo(t):
            f.write(" %08X(%s)" % (x.frm, idc.get_func_name(x.frm)))
        f.write("\n")
print("ok")
