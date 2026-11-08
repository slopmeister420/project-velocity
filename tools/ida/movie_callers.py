import idc, idautils, ida_funcs
out=open(r"D:/projects/project poland rewrite/research/movie_callers.tsv","w")
for l in open(r"D:/projects/project poland rewrite/research/movie_names.tsv"):
    a=int(l.split()[0],16)
    cs=set();d=0
    for r in idautils.XrefsTo(a):
        f=ida_funcs.get_func(r.frm)
        if f and r.type in (16,17,18,19,20,21): cs.add(f.start_ea)
        elif not f: d+=1
        elif r.type not in (16,17,18,19,20,21): d+=1
    out.write("%X\t%s\t%d\n"%(a,",".join("%X"%c for c in sorted(cs)),d))
out.close()
