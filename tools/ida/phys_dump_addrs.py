# physics agent: decompile the functions listed in research/phys_addrs.txt (hex, one per line) into
# research/phys_dump/Extra.c (keeps the functions already there)
import os, re, idc, ida_hexrays, ida_name, ida_funcs

ROOT = r"D:\projects\project poland rewrite\research"
path = os.path.join(ROOT, "phys_dump", os.environ.get("PHYS_DUMP_FILE", "Redump.c"))
have = {}
if os.path.exists(path):
    t = open(path, encoding="utf-8").read()
    for m in re.finditer(r"(?m)^// ==== ([0-9A-F]{8}) (\S+)\n(.*?)(?=^// ==== |\Z)", t, re.S):
        have[int(m.group(1), 16)] = (m.group(2), m.group(3))
for line in open(os.path.join(ROOT, "phys_addrs.txt")):
    line = line.split("#")[0].strip()
    if not line:
        continue
    a = int(line, 16)
    f = ida_funcs.get_func(a)
    if not f:
        print("no func", hex(a))
        continue
    try:
        have[f.start_ea] = (ida_name.get_name(f.start_ea), str(ida_hexrays.decompile(f.start_ea, flags=ida_hexrays.DECOMP_NO_CACHE)) + "\n")
    except Exception as ex:
        print("fail", hex(a), ex)
with open(path, "w", encoding="utf-8", newline="\n") as o:
    for a in sorted(have):
        n, b = have[a]
        o.write("// ==== %08X %s\n%s\n" % (a, n, b.strip("\n")))
print("Extra.c", len(have))
