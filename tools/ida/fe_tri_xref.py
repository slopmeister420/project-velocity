import idautils, idc
out = open(r"D:/projects/project poland rewrite/research/fe_tri_xref.txt", "w")
for t in (0x2515D0, 0x251550, 0x2514E0, 0x27E920, 0x250EE0, 0x250C20):
    for x in idautils.XrefsTo(t):
        out.write("%X <- %X (%s)\n" % (t, x.frm, idc.get_func_name(x.frm)))
out.close()
