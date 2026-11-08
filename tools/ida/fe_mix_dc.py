import ida_hexrays
out=[]
for a in (0x25E170,0x25E210,0x25E250,0x24D9E0,0x187BA0,0x187880):
    try: out.append("// ==== %X\n%s"%(a,ida_hexrays.decompile(a)))
    except Exception as e: out.append("// %X failed %s"%(a,e))
open(r"D:\projects\project poland rewrite\research\fe_mix_dc.c","w").write("\n".join(out))
