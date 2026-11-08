import idc, idaapi
out=open('D:/projects/project poland rewrite/research/gfx/s11_names.txt','w')
for i in range(6):
    p=idc.get_wide_dword(0x6867FC+4*i)
    out.write('%d %x %s\n'%(i,p,idc.get_strlit_contents(p)))
