import idautils, idc, ida_hexrays
out = []
for ea in idautils.Functions(0x00057fd0, 0x0005d0b0):
    try:
        c = str(ida_hexrays.decompile(ea))
    except Exception as e:
        c = 'ERR %s' % e
    out.append('//==== %x %s\n%s\n' % (ea, idc.get_func_name(ea), c))
open(r'D:/projects/project poland rewrite/research/core_vsrender.c', 'w').write('\n'.join(out))
