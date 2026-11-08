import idc, ida_funcs, ida_hexrays, idautils
# decompile the functions listed (hex per line) in research/audio/dec_req.txt -> research/audio/dec_out.c
B = r"D:\projects\project poland rewrite\research\audio"
ida_hexrays.init_hexrays_plugin()
with open(B + r"\dec_out.c", "w", encoding="utf-8") as fo:
    for line in open(B + r"\dec_req.txt"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        ea = int(line, 16)
        f = ida_funcs.get_func(ea)
        if not f:
            fo.write("// no func at %x\n" % ea)
            continue
        err = ida_hexrays.hexrays_failure_t()
        c = ida_hexrays.decompile(f.start_ea, err)
        if c is None:
            c = "// asm (%s)\n" % err.desc() + "\n".join("%X  %s" % (h, idc.generate_disasm_line(h, 0)) for h in idautils.FuncItems(f.start_ea))
        fo.write("// ===== %x %s\n%s\n\n" % (f.start_ea, idc.get_func_name(f.start_ea), c))
