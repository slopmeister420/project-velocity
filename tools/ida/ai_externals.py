# list functions outside the ai regions that ai code calls, with the calling ai files (ai agent)
import idautils, idc, ida_funcs, ida_name, bisect
OUT = r"D:\projects\project poland rewrite\research\ai_dump\_externals.txt"
regions = []
for line in open(r"D:\projects\project poland rewrite\research\objregions.tsv"):
    s, e, f = line.rstrip("\n").split("\t")
    regions.append((int(s, 16), int(e, 16), f))
regions.sort()
starts = [r[0] for r in regions]
def region(ea):
    i = bisect.bisect_right(starts, ea) - 1
    if i >= 0 and regions[i][0] <= ea < regions[i][1]:
        return regions[i][2]
    return "?"
AI = [(0x12440, 0x43610), (0xE72C0, 0xE85A0), (0x16E980, 0x16F5B0)]
def in_ai(ea):
    return any(lo <= ea < hi for lo, hi in AI)
calls = {}
for lo, hi in AI:
    for fea in idautils.Functions(lo, hi):
        f = ida_funcs.get_func(fea)
        for h in idautils.Heads(f.start_ea, f.end_ea):
            if idc.print_insn_mnem(h) != "call":
                continue
            t = idc.get_operand_value(h, 0)
            if not (0x11000 <= t < 0x3DDD20) or in_ai(t):
                continue
            calls.setdefault(t, set()).add(region(fea).split("\\")[-1])
with open(OUT, "w") as o:
    for t in sorted(calls, key=lambda t: region(t)):
        o.write("%08X %-50s %-40s %s\n" % (t, ida_name.get_name(t), region(t).split("\\")[-1], ",".join(sorted(calls[t]))))
print(len(calls))
