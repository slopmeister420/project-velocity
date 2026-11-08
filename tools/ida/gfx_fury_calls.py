# calls in GameManager 0xB9010 between 0xB9A00 and 0xBA2A0 -> research/gfx/fury_gm_calls.txt
import idautils, idc
out = "D:/projects/project poland rewrite/research/gfx/fury_gm_calls.txt"
with open(out, "w", encoding="utf-8") as f:
    for ea in idautils.Heads(0xB9A00, 0xBA2A0):
        m = idc.print_insn_mnem(ea)
        if m == "call" or (m == "test" and "24h" in idc.GetDisasm(ea)) or (m == "and" and "ecx," in idc.GetDisasm(ea)):
            f.write("%08X %s\n" % (ea, idc.GetDisasm(ea)))
print("ok")
