# dump BinkCopyToBuffer's path to the yuv->rgb blitters: decompile + disassembly
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/fe_yuv_blit.txt"
eas = [0x0053F970, 0x00548980, 0x00548DB0, 0x00550FC0]
with open(out, "w", encoding="utf-8") as f:
    for ea in eas:
        fn = ida_funcs.get_func(ea)
        f.write("===== %08X %s\n" % (ea, idc.get_func_name(ea)))
        if not fn:
            f.write("  no func\n")
            continue
        try:
            f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
        except Exception as e:
            f.write("  decompile failed %s\n" % e)
        f.write("---- asm\n")
        for i in idautils.FuncItems(fn.start_ea):
            f.write("%08X %s\n" % (i, idc.generate_disasm_line(i, 0)))
    f.write("===== data 00548600..00548700\n")
    for a in range(0x00548600, 0x00548700, 16):
        f.write("%08X %s\n" % (a, " ".join("%08X" % idc.get_wide_dword(a + k) for k in range(0, 16, 4))))
print("wrote", out)
