# frontend agent: static-init writers of string-crc tables -> research/fe_tbl.txt
import idc, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_tbl.txt", "w")
for base, n in [(0x6F0DA8, 12), (0x76A4A0, 2), (0x76A4A8, 3), (0x76A4B4, 3), (0x76A4C0, 3), (0x76A4CC, 1), (0x76A4D8, 5)]:
    for i in range(n):
        g = base + 4 * i
        for x in idautils.DataRefsTo(g):
            ins = idc.GetDisasm(x)
            if not ins.startswith("mov") or "offset" in ins:
                continue
            # look back for the CCRCChecked string
            p = x
            s = ""
            for _ in range(12):
                p = idc.prev_head(p)
                d = idc.GetDisasm(p)
                if "offset a" in d or '"' in d:
                    s = d
                    break
            out.write("%08X [%d] <- %08X %s | %s\n" % (base, i, x, ins, s))
out.close()
