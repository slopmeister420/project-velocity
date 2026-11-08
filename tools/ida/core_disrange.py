import idc, idautils
# disassemble ranges listed in research/core_disrange.in ("start end" hex per line) into research/core_disrange.txt
out = []
for line in open(r"D:/projects/project poland rewrite/research/core_disrange.in"):
    p = line.split()
    if len(p) < 2:
        continue
    s, e = int(p[0], 16), int(p[1], 16)
    out.append("== %X..%X" % (s, e))
    for h in idautils.Heads(s, e):
        out.append("%08X  %s" % (h, idc.GetDisasm(h)))
open(r"D:/projects/project poland rewrite/research/core_disrange.txt", "w").write("\n".join(out))
