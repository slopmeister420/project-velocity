import ida_hexrays, ida_funcs, idc, idautils
out = open(r"D:/projects/project poland rewrite/research/core_s8/ui.c", "w")
for a in [0x1E6A9F,0x1E6AC0,0x1E6AE0,0xB8280]:
    f=ida_funcs.get_func(a)
    if f: out.write("//==== %X %s\n%s\n" % (f.start_ea, ida_funcs.get_func_name(f.start_ea), str(ida_hexrays.decompile(f.start_ea))))
for a in range(0x1E6A90,0x1E6B00):
    f=ida_funcs.get_func(a)
    if f and f.start_ea==a: out.write("fn %X %s callers %s\n"%(a,ida_funcs.get_func_name(a),[hex(x.frm) for x in idautils.XrefsTo(a)][:8]))
out.close()
