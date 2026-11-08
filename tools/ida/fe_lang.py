# frontend agent: language screen tables + loop disasm
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_lang.txt", "w")
for base in (0x7609F0, 0x760A0C, 0x760A28):
    for i in range(7):
        g = base + 4*i
        for x in idautils.XrefsTo(g):
            p = x.frm; name = None
            for _ in range(12):
                p = idc.prev_head(p); d = idc.GetDisasm(p)
                if "push    offset" in d: name = d; break
            out.write("[%08X+%d] <- %08X %s | %s\n" % (base, i, x.frm, name, idc.GetDisasm(x.frm)))
ea = 0x2312D0
while ea < 0x231520:
    d = idc.GetDisasm(ea)
    if 0x231400 <= ea: out.write("%08X %s\n" % (ea, d))
    ea = idc.next_head(ea)
out.close()
