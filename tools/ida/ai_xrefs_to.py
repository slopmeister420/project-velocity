import idautils, idc
# callers of the addresses in research/ai_names_in.txt (hex) into research/ai_names_out.txt
base = r"D:\projects\project poland rewrite\research"
out = []
for l in open(base + r"\ai_names_in.txt"):
    l = l.strip()
    if not l:
        continue
    a = int(l, 16)
    for x in idautils.CodeRefsTo(a, 0):
        out.append("%X <- %X %s" % (a, x, idc.get_func_name(x)))
open(base + r"\ai_names_out.txt", "w").write("\n".join(out))
