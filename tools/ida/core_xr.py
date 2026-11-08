import idautils, idc
# callers of each address in research/core_dec_in.txt -> research/core_xr.txt
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_xr.txt", "w")
for line in open(base + r"\core_dec_in.txt"):
    s = line.strip()
    if not s:
        continue
    t = int(s, 16)
    out.write("== %x %s\n" % (t, idc.get_func_name(t)))
    for x in idautils.CodeRefsTo(t, 0):
        out.write("  %x in %s\n" % (x, idc.get_func_name(x)))
out.close()
