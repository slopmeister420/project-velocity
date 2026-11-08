import idc, ida_name, os
base = os.path.dirname(os.path.abspath(__file__))
out=[]
for n in open(os.path.join(base,"gfx_name_args.txt")).read().split():
    out.append("%s %08X" % (n, ida_name.get_name_ea(idc.BADADDR, n)))
open(os.path.join(base,"..","..","research","gfx","name.out"),"w").write("\n".join(out)+"\n")
print("ok")
