# rain surface / stencil-8 writers: decompile + list 0x2A1140 (stencil ref) callers with their pushed value
import ida_hexrays, idc, idautils, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx"
ADDRS = [0x136C50, 0x162280, 0x147DC0]
with open(OUT + r"\pud3_dec.c", "w", encoding="utf-8") as f:
    for a in ADDRS:
        f.write("//==== %08X %s\n" % (a, idc.get_func_name(a)))
        f.write("// callers: %s\n" % " ".join("%08X(%s)" % (ida_funcs.get_func(x).start_ea, idc.get_func_name(x)) for x in idautils.CodeRefsTo(a, 0) if ida_funcs.get_func(x)))
        try:
            f.write(str(ida_hexrays.decompile(a)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
with open(OUT + r"\pud3_stencil.txt", "w", encoding="utf-8") as f:
    for tgt in (0x2A1140, 0x2A10C0):
        for x in idautils.CodeRefsTo(tgt, 0):
            # walk back to the previous push
            ea = x
            val = "?"
            for _ in range(6):
                ea = idc.prev_head(ea)
                if idc.print_insn_mnem(ea) == "push":
                    val = idc.print_operand(ea, 0)
                    break
            f.write("%X call %X from %08X %s value %s\n" % (tgt, x, ida_funcs.get_func(x).start_ea if ida_funcs.get_func(x) else 0, idc.get_func_name(x), val))
print("ok")
