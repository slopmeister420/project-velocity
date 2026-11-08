import idautils
rows=[]
for s in idautils.Strings():
    rows.append("%08x\t%s" % (s.ea, str(s).replace("\n","\n").replace("\t","\t")))
open(r"D:\projects\project poland rewrite\research\strings.tsv","w",encoding="utf-8").write("\n".join(rows))
print(len(rows))
