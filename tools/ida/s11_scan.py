import idc, idautils, idaapi
out=open('D:/projects/project poland rewrite/research/gfx/s11_scan.txt','w')
for f in idautils.Functions(0x10F000,0x11E000):
    for h in idautils.FuncItems(f):
        d=idc.generate_disasm_line(h,0)
        if ' 77Ch' in d or '+77Ch' in d or '+8A8h' in d or '+8A0h' in d or ' 0DCh]' in d and False:
            out.write('%x %x %s\n'%(f,h,d))
