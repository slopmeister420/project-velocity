import idc, idautils, ida_hexrays, ida_bytes
# physics AddTweaks registrations (0x19BEC0 Materials, 0x1B8A00 JumpCam, 0x1DC650 Debug Cameras): decompile each
# and its callers into research/phys_tweak_regs_dc.c, plus the initial dwords of every global they reference
out = open(r"D:\projects\project poland rewrite\research\phys_tweak_regs_dc.c", "w", encoding="utf-8")
for f in [0x19BEC0, 0x1B8A00, 0x1DC650]:
    out.write("\n==== %x %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n")
    except Exception as ex:
        out.write("ERR %s\n" % ex)
    for x in idautils.XrefsTo(f):
        out.write("// caller %X in %s\n" % (x.frm, idc.get_func_name(x.frm)))
out.close()
print("ok")
