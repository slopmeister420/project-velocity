# frontend agent: FECK2Element2DLine functions -> research/fe_line.c
import idc, ida_hexrays, idautils
out = open(r"D:\projects\project poland rewrite\research\fe_line.c", "w", encoding="utf-8")
for f in idautils.Functions(0x293300, 0x293A00):
    out.write("// ===== %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n\n")
    except Exception as ex:
        out.write("// failed %s\n" % ex)
# the call in 0x241940
ea = 0x241940
while ea < 0x241A10:
    out.write("// %08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x241A10)
# vtable refs to the 2D line ctor's vtable
for x in idautils.XrefsTo(0x293670):
    out.write("// xref 293670 from %08X\n" % x.frm)
out.close()
