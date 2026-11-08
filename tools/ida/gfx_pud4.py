# data xrefs of the puddle globals (colour, shader select, mist vectors) -> research/gfx/pud4_xref.txt
import idc, idautils, ida_funcs, ida_bytes
OUT = r"D:\projects\project poland rewrite\research\gfx\pud4_xref.txt"
ADDRS = [0x721B34, 0x72639B, 0x721B24, 0x721B38, 0x721B48, 0x7207CC, 0x72081C, 0x721B14, 0x72082C, 0x7207C8]
with open(OUT, "w") as f:
    for a in ADDRS:
        f.write("%X init %08X\n" % (a, ida_bytes.get_dword(a)))
        for x in idautils.DataRefsTo(a):
            fn = ida_funcs.get_func(x)
            f.write("  %08X %s %s | %s\n" % (x, "%08X" % fn.start_ea if fn else "-", idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
print("ok")
