import ida_hexrays, ida_funcs
f = ida_funcs.get_func(0x000F82F0)
c = ida_hexrays.decompile(f.start_ea)
open(r"D:/projects/project poland rewrite/research/fe_osd_create_f82f0.c", "w").write(str(c))
