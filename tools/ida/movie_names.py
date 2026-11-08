import idc, idautils, ida_funcs
out=open(r"D:/projects/project poland rewrite/research/movie_names.tsv","w")
for f in idautils.Functions(0x2f3160,0x2f6710):
    out.write("%08X\t%d\t%s\n"%(f,ida_funcs.get_func(f).size(),idc.get_func_name(f)))
for f in idautils.Functions(0x53E0C0,0x551938):
    out.write("%08X\t%d\t%s\n"%(f,ida_funcs.get_func(f).size(),idc.get_func_name(f)))
out.close()
