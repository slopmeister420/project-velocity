import idc
# disassembly before each FuryFX 0x00128680 / 0x00154FC0 camera call + ZoneRender 0x162D10 tail -> research/core_fury_calls.txt
out = open(r"D:\projects\project poland rewrite\research\core_fury_calls.txt", "w")
for site in (0x66C5E, 0x67E54, 0x68801, 0x691D9, 0x162D64, 0x1287B0):
    out.write("== %x\n" % site)
    ea = site
    for _ in range(14):
        ea = idc.prev_head(ea)
    for _ in range(22):
        out.write("  %x %s\n" % (ea, idc.GetDisasm(ea)))
        ea = idc.next_head(ea)
out.close()
