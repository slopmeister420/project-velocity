# frontend agent: apply renames/prototypes from fe_rename.tsv (addr<TAB>name[<TAB>prototype])
import idc, ida_name, ida_hexrays
n = 0
for line in open(r"D:\projects\project poland rewrite\tools\ida\fe_rename.tsv", encoding="utf-8"):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    p = line.split("\t")
    ea = int(p[0], 16)
    if len(p) > 1 and p[1]:
        if not idc.set_name(ea, p[1], ida_name.SN_NOWARN | ida_name.SN_NOCHECK | ida_name.SN_FORCE):
            print("rename failed", hex(ea), p[1])
    if len(p) > 2 and p[2]:
        if not idc.SetType(ea, p[2]):
            print("type failed", hex(ea), p[2])
        ida_hexrays.mark_cfunc_dirty(ea)
    n += 1
print("applied", n)
