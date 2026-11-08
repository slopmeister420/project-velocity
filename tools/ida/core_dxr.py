import idautils, idc
# data refs to each address in research/core_dec_in.txt -> research/core_xr.txt (with the instruction text)
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_xr.txt", "w")
for line in open(base + r"\core_dec_in.txt"):
    s = line.strip()
    if not s:
        continue
    t = int(s, 16)
    out.write("== %x\n" % t)
    for x in idautils.DataRefsTo(t):
        out.write("  %x %s | %s\n" % (x, idc.get_func_name(x), idc.generate_disasm_line(x, 0)))
out.close()
