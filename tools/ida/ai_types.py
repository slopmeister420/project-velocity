# apply research/ai_dump/_types.txt ("hexaddr|c declaration;" per line) to the idb (ai agent)
import idc, ida_name
ok = bad = 0
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_types.txt"):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    a, decl = line.split("|", 1)
    ea = int(a, 16)
    name = ida_name.get_name(ea)
    safe = "".join(c if c.isalnum() or c == "_" else "_" for c in name)
    decl = decl.replace("FN", safe)
    if idc.SetType(ea, decl):
        ok += 1
    else:
        bad += 1
        print("fail", hex(ea), decl)
print("typed", ok, "failed", bad)
