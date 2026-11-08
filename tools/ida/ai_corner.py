import ida_hexrays, idautils, idc, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\ai_dump\corner.c", "w")
def dec(ea):
    try:
        out.write("// ==== %X\n%s\n\n" % (ea, ida_hexrays.decompile(ea)))
    except Exception as e:
        out.write("// ==== %X failed %s\n" % (ea, e))
for ea in [0x2C1B0, 0x15330, 0x23C20, 0x21DD0, 0x1C600]:
    dec(ea)
for g in [0x6D3880, 0x6D3814, 0x6D36C8]:
    out.write("// xrefs to %X (value %08X): %s\n" % (g, idc.get_wide_dword(g),
        " ".join("%X:%s" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(g))))
out.close()
