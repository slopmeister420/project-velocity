import ida_hexrays
# decompile the function containing each address in ai_ctx_in.txt ("addr window"), print lines near any call
# whose text contains the needle -> ai_ctx_out.txt
base = r"D:\projects\project poland rewrite\research"
out = []
for l in open(base + r"\ai_ctx_in.txt"):
    if not l.strip():
        continue
    a, needle, win = l.split()
    a = int(a, 16); win = int(win)
    try:
        lines = str(ida_hexrays.decompile(a)).split("\n")
    except Exception as e:
        out.append("fail %X %s" % (a, e)); continue
    for i, t in enumerate(lines):
        if needle in t:
            out.append("== %X line %d" % (a, i))
            out.extend(lines[max(0, i - win):i + win + 1])
open(base + r"\ai_ctx_out.txt", "w").write("\n".join(out))
