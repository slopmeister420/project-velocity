import idautils, idc
# callers of each address in research/ai_callers_in.txt -> research/ai_callers_out.txt
base = r"D:\projects\project poland rewrite\research"
out = []
for l in open(base + r"\ai_callers_in.txt"):
    if not l.strip():
        continue
    a = int(l.strip(), 16)
    out.append("== %X %s" % (a, idc.get_func_name(a)))
    for x in idautils.CodeRefsTo(a, 0):
        out.append("  %X %s" % (x, idc.get_func_name(x)))
open(base + r"\ai_callers_out.txt", "w").write("\n".join(out))
