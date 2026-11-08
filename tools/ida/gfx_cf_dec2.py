# car light flares part 2 -> research/gfx/cf_dec2.c
import idc, ida_hexrays, idautils, ida_funcs
out = []
def dec(ea):
    f = ida_funcs.get_func(ea)
    ea = f.start_ea if f else ea
    try:
        out.append("//==== %X %s" % (ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
def dis(a, b):
    out.append("//---- dis %X..%X" % (a, b))
    ea = a
    while ea < b:
        out.append("%06X %s" % (ea, idc.GetDisasm(ea)))
        ea = idc.next_head(ea, b + 1)
for ea in (0x4251AB, 0x4252BE, 0x4252F7, 0x425321, 0x425351, 0x425387, 0x4253B2, 0x4254BC):
    dec(ea)
for ea in (0x179F4, 0x2B4B0, 0x2B5D0, 0x17A490):
    dec(ea)
dis(0x124270, 0x1242E0)
dis(0x19AD0, 0x19B00)
dis(0xBA250, 0xBA280)
dis(0x299C30, 0x299C60)
dis(0x119830, 0x1198B0)
dis(0xB7040, 0xB7060)
for i in range(2):
    p = idc.get_wide_dword(0x686770 + 4 * i)
    out.append("off_686770[%d] = %X %r" % (i, p, idc.get_strlit_contents(p)))
seen = set()
for i in range(0, 12):
    pass
open(r"D:\projects\project poland rewrite\research\gfx\cf_dec2.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
