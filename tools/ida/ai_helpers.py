# list small ai-region helpers with call counts and one-line bodies (ai agent)
import idautils, idc, ida_funcs, ida_hexrays, re
RANGES = [(0x12440, 0x43610)]
OUT = r"D:\projects\project poland rewrite\research\ai_dump\_helpers.txt"
out = []
for lo, hi in RANGES:
    for fea in idautils.Functions(lo, hi):
        f = ida_funcs.get_func(fea)
        if f.end_ea - f.start_ea > 0xC0:
            continue
        n = sum(1 for x in idautils.XrefsTo(fea) if x.iscode)
        if n < 3:
            continue
        try:
            s = str(ida_hexrays.decompile(fea, flags=ida_hexrays.DECOMP_NO_CACHE))
        except Exception:
            continue
        lines = [l.strip() for l in s.splitlines()]
        body = " ".join(l for l in lines[1:] if l and not l.startswith("//") and l not in ("{", "}"))
        body = re.sub(r"\s+", " ", body)
        out.append((n, fea, lines[0], body[:300]))
out.sort(reverse=True)
with open(OUT, "w") as o:
    for n, fea, sig, body in out:
        o.write("%4d %08X %s | %s\n" % (n, fea, sig, body))
