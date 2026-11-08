# frontend agent: byte_769380 (area background offset) value + xrefs, 0x273700 body -> research/fe_769380.txt
import idautils, idc, ida_bytes, struct, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\fe_769380.txt", "w")
b = ida_bytes.get_bytes(0x769380, 16)
out.write("value %s\n" % (struct.unpack("<4f", b) if b else None,))
for x in idautils.XrefsTo(0x769380, 0):
    out.write("  %08X %s  %s\n" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
out.write(str(ida_hexrays.decompile(0x273700)))
out.close()
