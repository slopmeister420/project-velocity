import ida_hexrays, ida_funcs
f = ida_funcs.get_func(0x19E080)
c = ida_hexrays.decompile(f)
lines = str(c).split("\n")
out = open(r"D:/projects/project poland rewrite/research/core_physmgr_19e080.c", "w")
out.write("\n".join(lines))
out.close()
print(len(lines))
