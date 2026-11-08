import idc, idautils, ida_funcs, ida_bytes, ida_hexrays
# review tweak, jump cam option byte, and the 0x1CCAD0 replay manager calls
base = r"D:\projects\project poland rewrite\research"
out = open(base + r"\core_attract5.txt", "w")
for ea in (0x63EF00, 0x75B278, 0x62ED04, 0x749114, 0x749118, 0x749120):
    out.write("== xrefs %x (byte %x)\n" % (ea, ida_bytes.get_byte(ea)))
    for x in idautils.XrefsTo(ea):
        out.write("  %x %s\n" % (x.frm, idc.get_func_name(x.frm)))
for ea in (0x1CD0E0, 0x1CD0F0, 0x1CD100, 0x1CD120, 0x1CD130, 0x1CD140):
    pass
ea = 0x1CD0B0
while ea < 0x1CD180:
    out.write("%x  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x1CD180)
# the tweak registration context
for x in idautils.XrefsTo(0x63EF00):
    ea = x.frm - 0x30
    while ea < x.frm + 0x30:
        out.write("%x  %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea, x.frm + 0x30)
out.close()
