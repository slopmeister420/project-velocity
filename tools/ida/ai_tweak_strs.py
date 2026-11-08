import idc, idautils
# strings behind the AddTweaks name pointers + callers of the TFT tweak registration
out = []
for a in (0x624DF8, 0x624E10, 0x626350, 0x626334, 0x626318):
    p = idc.get_wide_dword(a)
    s = idc.get_strlit_contents(p, -1, 0)
    s2 = idc.get_strlit_contents(a, -1, 0)
    out.append("%X -> %X %r | direct %r" % (a, p, s, s2))
for f in (0x16C30, 0x2D360):
    for x in idautils.CodeRefsTo(f, 0):
        out.append("caller of %X: %X %s" % (f, x, idc.get_func_name(x)))
for a in (0x684E58, 0x684E5C, 0x684E60, 0x684E54):
    out.append("%X = %08X" % (a, idc.get_wide_dword(a)))
open(r"D:\projects\project poland rewrite\research\ai_dump\_tweak_strs.txt", "w").write("\n".join(out))
