# frontend agent: RaceCompleteScreen save prompts (XMem) -> research/fe_xmem.c
import idc, ida_hexrays, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_xmem.c", "w", encoding="utf-8")
def dc(f):
    f = idc.get_func_attr(f, idc.FUNCATTR_START)
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
for g in [0x6F9054, 0x701404, 0x6F7165]:
    out.write("// xrefs %08X\n" % g)
    for x in idautils.XrefsTo(g):
        f = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("//   %08X %s %s\n" % (x.frm, idc.get_func_name(f), idc.generate_disasm_line(x.frm, 0)))
for f in [0x30EF60, 0x30EF50, 0x1061D0]:
    out.write("// xrefs to %08X\n" % f)
    for x in idautils.XrefsTo(f):
        g = idc.get_func_attr(x.frm, idc.FUNCATTR_START)
        out.write("//   %08X %s\n" % (x.frm, idc.get_func_name(g)))
for f in [0x30EF60, 0x30EF50, 0x1061D0, 0x27AD30, 0x27B980, 0x27B8E0, 0x27BA70, 0x27B200, 0x27B440]:
    dc(f)
out.close()
