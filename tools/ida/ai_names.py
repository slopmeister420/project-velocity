import idc
# print addresses of names listed in research/ai_names_in.txt into research/ai_names_out.txt
base = r"D:\projects\project poland rewrite\research"
out = []
for l in open(base + r"\ai_names_in.txt"):
    n = l.strip()
    if n:
        out.append("%s %X" % (n, idc.get_name_ea_simple(n)))
open(base + r"\ai_names_out.txt", "w").write("\n".join(out))
