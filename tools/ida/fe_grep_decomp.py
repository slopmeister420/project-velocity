# frontend agent: decompile functions in [start,end) ranges from fe_grep_args.txt and print lines matching a regex
import re, idautils, ida_hexrays, idc
lines = open(r"D:\projects\project poland rewrite\tools\ida\fe_grep_args.txt").read().split("\n")
pat = re.compile(lines[0])
for l in lines[1:]:
    p = l.split()
    if len(p) < 2:
        continue
    for f in idautils.Functions(int(p[0], 16), int(p[1], 16)):
        try:
            s = str(ida_hexrays.decompile(f))
        except Exception:
            continue
        for ln in s.split("\n"):
            if pat.search(ln):
                print("%08X %s: %s" % (f, idc.get_func_name(f), ln.strip()))
