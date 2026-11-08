# physics agent: retype decompiler locals listed in research/phys_lvars.txt ("ADDR name C-type", e.g.
# "1CB530 v18 int[24]") and re-decompile those functions into research/phys_dump/Redump.c
import os, re, idc, ida_hexrays, ida_name, ida_typeinf, ida_funcs

ROOT = r"D:\projects\project poland rewrite\research"
done = []
for line in open(os.path.join(ROOT, "phys_lvars.txt")):
    line = line.split("#")[0].strip()
    if not line:
        continue
    a, name, ty = line.split(None, 2)
    ea = int(a, 16)
    tif = ida_typeinf.tinfo_t()
    m = re.match(r"(.*)\[(\d+)\]$", ty)
    if m:
        el = ida_typeinf.tinfo_t()
        ida_typeinf.parse_decl(el, None, m.group(1) + " x;", ida_typeinf.PT_SIL)
        tif.create_array(el, int(m.group(2)))
    else:
        ida_typeinf.parse_decl(tif, None, ty + " x;", ida_typeinf.PT_SIL)
    info = ida_hexrays.lvar_saved_info_t()
    cf = ida_hexrays.decompile(ea)
    ok = False
    for lv in cf.get_lvars():
        if lv.name == name:
            info.ll = lv
            info.type = tif
            ok = ida_hexrays.modify_user_lvar_info(ea, ida_hexrays.MLI_TYPE, info)
    print("%X %s %s %s" % (ea, name, ty, "ok" if ok else "FAILED"))
    if ok and ea not in done:
        done.append(ea)
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
