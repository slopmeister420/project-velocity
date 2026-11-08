# xrefs (with containing function) to the SpecularChars globals; decompile the functions that touch the resolver data
import idautils, idc, ida_funcs, ida_hexrays, ida_bytes, struct
out = r"D:/projects/project poland rewrite/research/gfx/spec_xr.txt"
addrs = [0x6D6D58, 0x6D6DB0, 0x6D6DAC, 0x6D6D9C, 0x6D6D5C, 0x736530, 0x736500, 0x6D6CC8, 0x6D6CCC, 0x1442F0, 0x145F20, 0x143A40, 0x6D6DBC]
funcs = set()
with open(out, "w", encoding="utf-8") as f:
    for a in addrs:
        f.write("== xrefs to %08X %s\n" % (a, idc.get_name(a)))
        for x in idautils.XrefsTo(a):
            fn = ida_funcs.get_func(x.frm)
            nm = idc.get_func_name(fn.start_ea) if fn else "?"
            f.write("  %08X %s  %s\n" % (x.frm, nm, idc.GetDisasm(x.frm)))
            if fn and a in (0x6D6DB0, 0x6D6DAC, 0x6D6D58):
                funcs.add(fn.start_ea)
    # names of material strings
    for a in (0x63D9B4, 0x63D994):
        f.write("str %08X %r\n" % (a, idc.get_strlit_contents(a)))
    # dword at 6D6D58 region (vtable?)
    for fe in sorted(funcs):
        if 0x143000 <= fe < 0x147000 or 0x79000 <= fe < 0x7B000:
            continue
        f.write("\n//==== %08X %s\n" % (fe, idc.get_func_name(fe)))
        try:
            f.write(str(ida_hexrays.decompile(fe)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("wrote", out)
