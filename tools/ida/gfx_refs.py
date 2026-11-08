import idautils, idc
out=[]
for ea in range(0x705F20, 0x705F40, 4):
    for x in idautils.XrefsTo(ea):
        out.append('%X <- %X %s | %s' % (ea, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
print('\n'.join(out))
