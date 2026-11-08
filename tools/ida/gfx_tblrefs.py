import idautils, idc, ida_xref
lo, hi = 0x706260, 0x706360
out = []
for ea in range(lo, hi, 4):
    for x in idautils.XrefsTo(ea):
        out.append('%X <- %X %s | %s' % (ea, x.frm, idc.get_func_name(x.frm), idc.generate_disasm_line(x.frm, 0)))
open(r'D:/projects/project poland rewrite/research/gfx2/tblrefs.txt', 'w').write('\n'.join(out))
print(len(out))
