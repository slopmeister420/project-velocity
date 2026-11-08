# frontend agent: static-init writes into the action->string table 0x688B98
import idautils, idc
out = open(r"D:\projects\project poland rewrite\research\fe_opt_q3.txt", "w")
for g in range(0x688B98, 0x688C10, 4):
    for x in idautils.XrefsTo(g):
        p = x.frm; name = None
        for _ in range(12):
            p = idc.prev_head(p)
            d = idc.GetDisasm(p)
            if "push    offset" in d:
                name = d; break
        out.write("%08X <- %08X %s | %s | prev %s\n" % (g, x.frm, name, idc.GetDisasm(x.frm), idc.GetDisasm(idc.prev_head(x.frm))))
out.close()
