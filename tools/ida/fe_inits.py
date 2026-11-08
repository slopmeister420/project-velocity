# frontend agent: static-init string crcs stored into the given globals (fe_peek_args.txt: start end)
import idautils, idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
s, e = int(a[0], 16), int(a[1], 16)
for g in range(s, e, 4):
    for x in idautils.XrefsTo(g):
        if x.frm < 0x3DDD20:
            continue
        p = x.frm
        name = None
        for _ in range(8):
            p = idc.prev_head(p)
            d = idc.GetDisasm(p)
            if "push    offset" in d and ";" in d:
                name = d.split(";", 1)[1].strip()
                break
        print("%08X <- %08X %s  |  %s" % (g, x.frm, name, idc.GetDisasm(x.frm)))
