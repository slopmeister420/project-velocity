import idautils, idc, ida_funcs
OUT = r"D:\projects\project poland rewrite\research\gfx\fse_xrefs.txt"
eas = [0x6308e4,0x6308f8,0x631220,0x63b1e4,0x63bd54,0x63c234,0x63c488,0x63c4a4,0x63c4b0,0x63c4c0,0x63c500,
       0x63c9d8,0x63d7dc,0x63d7f0,0x63e78c,0x63e798,0x63e850,0x63e860,0x63e874,0x63e890,0x63e8a4,0x63e8b8,
       0x63e970,0x63e988,0x63e994,0x63e9a0,0x63e9b4,0x63eba8,0x63ebb8,0x63ebf8,0x63ec18,0x63ec30,0x63ed8c]
with open(OUT, "w") as o:
    for ea in eas:
        for x in idautils.XrefsTo(ea):
            f = ida_funcs.get_func(x.frm)
            o.write("%x %s %x %s\n" % (ea, idc.get_strlit_contents(ea), x.frm, f and idc.get_func_name(f.start_ea)))
print("ok")
