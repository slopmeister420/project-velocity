import idautils, idc, ida_funcs, ida_hexrays
out = open(r"D:/projects/project poland rewrite/research/gfxdmg/locators.c", "w")
done = set()
for ea in [0x63b324, 0x63b32c]:
    for x in idautils.XrefsTo(ea):
        f = ida_funcs.get_func(x.frm)
        out.write("// xref %x from %x %s\n" % (ea, x.frm, f and idc.get_func_name(f.start_ea)))
        if f and f.start_ea not in done:
            done.add(f.start_ea)
            out.write(str(ida_hexrays.decompile(f.start_ea)) + "\n")
print("ok", len(done))
