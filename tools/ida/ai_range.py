import idc
# disasm ranges from research/ai_dump/_rng.txt ("start end" hex) -> _rng_out.txt (ai agent)
o = open(r"D:\projects\project poland rewrite\research\ai_dump\_rng_out.txt", "w")
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_rng.txt"):
    if not line.strip():
        continue
    s, e = (int(t, 16) for t in line.split())
    x = s
    while x < e:
        o.write("%X  %s\n" % (x, idc.GetDisasm(x)))
        x = idc.next_head(x)
print("ok")
