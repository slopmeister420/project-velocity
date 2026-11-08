# callers of Characters 0x79EC0 / 0x7A460 and of CarDamageFX 0x79C30: decompile each caller to a file
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/spec_callers.c"
done = set()
with open(out, "w", encoding="utf-8") as f:
    for t in (0x79EC0, 0x7A460, 0x79C30, 0x62B0D8):
        for x in idautils.XrefsTo(t):
            fn = ida_funcs.get_func(x.frm)
            f.write("// xref %08X -> %08X in %s\n" % (x.frm, t, idc.get_func_name(fn.start_ea) if fn else "?"))
            if not fn or fn.start_ea in done:
                continue
            done.add(fn.start_ea)
            f.write("//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
            try:
                f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)
    import ida_bytes
    f.write("// vtable 62B0D8\n")
    for i in range(6):
        p = ida_bytes.get_dword(0x62B0D8 + 4 * i)
        f.write("//  [%d] %08X %s\n" % (i, p, idc.get_func_name(p)))
        fn = ida_funcs.get_func(p)
        if fn and fn.start_ea not in done and 0x11000 <= p < 0x3DDD20:
            done.add(fn.start_ea)
            f.write("//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
            try:
                f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
            except Exception as ex:
                f.write("// fail %s\n" % ex)
print("wrote", out)
