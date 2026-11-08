import idc, ida_hexrays, ida_bytes, struct, idautils
# name tag colour (VsAIDriver 0x15F80) -> research/ai_dump/name_colour.c
out = open(r"D:\projects\project poland rewrite\research\ai_dump\name_colour.c", "w")
for ea in [0x15F80, 0x156C0, 0x15ED0, 0x15470]:
    out.write("==== %X %s\n" % (ea, idc.get_func_name(ea)))
    try:
        out.write(str(ida_hexrays.decompile(ea)) + "\n")
    except Exception as e:
        out.write("ERR %s\n" % e)
    out.write("-- callers: %s\n" % ", ".join("%X %s" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(ea)))
out.close()
