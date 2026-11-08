# address of names listed in research/ai_dump/_req.txt (ai agent)
import ida_name
out = open(r"D:\projects\project poland rewrite\research\ai_dump\_xref.txt", "w")
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_req.txt"):
    n = line.strip()
    if n:
        out.write("%s %X\n" % (n, ida_name.get_name_ea(0, n)))
out.close()
