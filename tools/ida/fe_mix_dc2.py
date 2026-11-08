import ida_hexrays
out=[]
for a in (0x10D0B0,):
    out.append("// ==== %X\n%s"%(a,ida_hexrays.decompile(a)))
open(r"D:\projects\project poland rewrite\research\fe_mix_dc2.c","w").write("\n".join(out))
