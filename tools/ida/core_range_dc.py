import idc, ida_hexrays, ida_funcs, idautils, re
# decompile every function in the ranges of research/core_ranges.txt ("start end out" per line, hex) to research/<out>
base = r"D:\projects\project poland rewrite\research"
NOISE = re.compile(r"__debugbreak|^\s*[{}]\s*$|off_6C18D8 != nullptr|InputDevice__3DDC30\(1\);|memset\(.*0xCC|= off_6C18D8\(|if \( v[0-9]+ == 3 \)|(HI|LO)WORD\(v.*= 257;|BYTE[0-9]?\(v.*= 1;|^\s*$")
for line in open(base + r"\core_ranges.txt"):
    p = line.split()
    if len(p) < 3:
        continue
    s, e, name = int(p[0], 16), int(p[1], 16), p[2]
    out = open(base + "\\" + name, "w", encoding="utf-8")
    for f in idautils.Functions(s, e):
        out.write("\n==== %x %s\n" % (f, idc.get_func_name(f)))
        try:
            txt = str(ida_hexrays.decompile(f))
            out.write("\n".join(l for l in txt.split("\n") if not NOISE.search(l)) + "\n")
        except Exception as ex:
            out.write("ERR %s\n" % ex)
    out.close()
print("ok")
