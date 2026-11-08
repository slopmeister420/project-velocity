import idc, struct
out=open(r"D:\projects\project poland rewrite\research\g2_vals.txt","w")
for a in [0x6870B0,0x6870A0,0x6870AC,0x687054,0x687058,0x68705C,0x68706C,0x687060,0x687064,0x687068]:
    v=idc.get_wide_dword(a); out.write("%X %08X %r\n"%(a,v,struct.unpack('<f',struct.pack('<I',v))[0]))
out.close()
