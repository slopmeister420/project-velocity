# apply research/ai_dump/_renames.txt ("hexaddr name" per line) to the idb (ai agent)
import idc, ida_name
ok = bad = 0
for line in open(r"D:\projects\project poland rewrite\research\ai_dump\_renames.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    a, n = line.split(None, 1)
    ea = int(a, 16)
    cur = ida_name.get_name(ea)
    if cur == n:
        continue
    other = ida_name.get_name_ea(idc.BADADDR, n)
    if other != idc.BADADDR and other != ea:
        n2 = "%s_%X" % (n, ea)
        print("dup", n, "at", hex(other), "->", n2)
        n = n2
    if ida_name.set_name(ea, n, ida_name.SN_NOWARN | ida_name.SN_NOCHECK | ida_name.SN_FORCE):
        ok += 1
    else:
        bad += 1
        print("fail", hex(ea), n)
print("renamed", ok, "failed", bad)
