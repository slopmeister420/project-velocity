# frontend agent: keyboard overlay helpers -> research/fe_kb2.c
import idc, idautils, ida_hexrays, ida_bytes
out = open(r"D:\projects\project poland rewrite\research\fe_kb2.c", "w", encoding="utf-8")
def dc(ea):
    f = idc.get_func_attr(ea, idc.FUNCATTR_START)
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
for a in [0x268560, 0x268160, 0x26E700, 0x26E740]:
    dc(a)
out.write("// ---- static init 0x42F420..0x42F600\n")
ea = 0x42F420
while ea < 0x42F600:
    out.write("%08X  %s\n" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea)
out.write("// ---- ProcessEvent / Render around keyboard\n")
for s, e in [(0x283D40, 0x283DA0), (0x283F80, 0x283FC0)]:
    ea = s
    while ea < e:
        out.write("%08X  %s\n" % (ea, idc.GetDisasm(ea)))
        ea = idc.next_head(ea)
out.close()
