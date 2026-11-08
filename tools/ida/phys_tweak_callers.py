import idc, idautils
# who calls PhysicsManager 0x19E080 (-> VehicleManager 0x1DC650 -> PhysicsCarInput 0x1B8A00 tweaks)
out = open(r"D:\projects\project poland rewrite\research\phys_tweak_callers.txt", "w", encoding="utf-8")
for f in [0x19E080]:
    for x in idautils.XrefsTo(f):
        out.write("%X <- %X %s\n" % (f, x.frm, idc.get_func_name(x.frm)))
        for y in idautils.XrefsTo(idc.get_func_attr(x.frm, idc.FUNCATTR_START)):
            out.write("   %X <- %X %s\n" % (x.frm, y.frm, idc.get_func_name(y.frm)))
out.close()
print("ok")
