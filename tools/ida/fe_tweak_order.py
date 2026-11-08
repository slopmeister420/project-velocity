# frontend agent: call order of the frontend tweak-registering creates -> research/fe_tweak_order.txt
import idc, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_tweak_order.txt", "w", encoding="utf-8")
targets = [0x2181A0, 0x21C9C0, 0x23B370, 0x247990, 0x258690, 0x26F3D0, 0x1FDC20, 0x1FEB90]
def chain(t, depth):
    for x in idautils.XrefsTo(t):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("%s%X <- %X in %X %s\n" % ("  " * depth, t, x.frm, f if f != idc.BADADDR else 0, idc.get_func_name(x.frm)))
        if depth < 2 and f != idc.BADADDR:
            chain(f, depth + 1)
for t in targets:
    out.write("== %X %s\n" % (t, idc.get_func_name(t)))
    chain(t, 0)
out.close()
