import ida_hexrays, idautils, idc
out = open(r"D:/projects/project poland rewrite/research/fe_custom_region.c", "w")
for f in idautils.Functions(0x24F680, 0x2541F0):
    out.write("\n//==== 0x%X\n" % f)
    try:
        out.write(str(ida_hexrays.decompile(f)))
    except Exception as e:
        out.write("ERR %s\n" % e)
out.close()
