# puddle draw helpers: decompile listed functions + disassemble 0x131F50 into research/gfx/pud_*.txt
import ida_hexrays, ida_funcs, idc, idautils
OUT = r"D:\projects\project poland rewrite\research\gfx"
ADDRS = [0x43FB0, 0x42DD0, 0x42E00, 0x2B95D0, 0x2B8C70, 0x2B90A0, 0x2A9A80, 0x1338F0, 0x5E070, 0x5ABB0, 0x2AB250, 0xEFD30]
with open(OUT + r"\pud_dec.c", "w", encoding="utf-8") as f:
    for a in ADDRS:
        f.write("//==== %08X %s\n" % (a, idc.get_func_name(a)))
        try:
            f.write(str(ida_hexrays.decompile(a)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
with open(OUT + r"\pud_dis.txt", "w", encoding="utf-8") as f:
    fn = ida_funcs.get_func(0x131F50)
    for ea in idautils.Heads(fn.start_ea, fn.end_ea):
        f.write("%08X  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
print("ok")
