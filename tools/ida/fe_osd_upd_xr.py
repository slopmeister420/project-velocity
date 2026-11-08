# frontend agent: who runs COSDManager::Update / UpdateItem, and on which manager (dword_6FD1FC vs dword_6FD200)
import idc, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_osd_upd_xr.txt", "w", encoding="utf-8")
for t in [0xEE910, 0xEE5F0]:
    out.write("== %X %s\n" % (t, idc.get_func_name(t)))
    for x in idautils.XrefsTo(t):
        out.write("  from %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
        ea = x.frm
        for _ in range(6):
            ea = idc.prev_head(ea)
            out.write("      %X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
for g in [0x6FD1FC]:
    out.write("== xrefs %X\n" % g)
    for x in idautils.XrefsTo(g):
        out.write("  %X %s | %s\n" % (x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
out.close()
