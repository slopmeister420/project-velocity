# physics agent: apply prototypes listed in research/phys_types.txt ("ADDR<TAB>C declaration;") and
# re-decompile those functions into research/phys_dump/Redump.c
import os, re, idc, ida_hexrays, ida_name, ida_funcs

ROOT = r"D:\projects\project poland rewrite\research"
done = []
for line in open(os.path.join(ROOT, "phys_types.txt")):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    if "\t" not in line:
        done.append(int(line, 16))
        continue
    a, decl = line.split("\t", 1)
    ea = int(a, 16)
    ok = idc.SetType(ea, decl)
    print("%X %s %s" % (ea, "ok" if ok else "FAILED", decl))
    if ok:
        done.append(ea)
        # callers see the new prototype only after a re-decompile
        import idautils
        for x in idautils.CodeRefsTo(ea, 0):
            f = ida_funcs.get_func(x)
            if f and f.start_ea not in done:
                done.append(f.start_ea)
path = os.path.join(ROOT, "phys_dump", "Redump.c")
have = {}
if os.path.exists(path):
    t = open(path, encoding="utf-8").read()
    for m in re.finditer(r"(?m)^// ==== ([0-9A-F]{8}) (\S+)\n(.*?)(?=^// ==== |\Z)", t, re.S):
        have[int(m.group(1), 16)] = (m.group(2), m.group(3))
for ea in done:
    ida_hexrays.mark_cfunc_dirty(ea)
    have[ea] = (ida_name.get_name(ea), str(ida_hexrays.decompile(ea)) + "\n")
with open(path, "w", encoding="utf-8") as o:
    for a in sorted(have):
        n, b = have[a]
        o.write("// ==== %08X %s\n%s\n" % (a, n, b.strip("\n")))
print("Redump.c", len(have))
