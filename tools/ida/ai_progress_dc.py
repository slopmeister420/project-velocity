import idc, ida_hexrays, ida_name
# race progress getters -> research/ai_dump/progress_dc.c
out = open(r"D:\projects\project poland rewrite\research\ai_dump\progress_dc.c", "w")
names = ["VsAIWorld::GetVehicleInPosition", "VsAIWorld::GetRaceLength"]
addrs = [ida_name.get_name_ea(idc.BADADDR, n) for n in names] + [0x37D80, 0x244E0, 0x23CE0, 0x3DB70, 0x15D30, 0x19350, 0x15C20, 0x15DB0]
for ea in addrs:
    out.write("==== %X %s\n" % (ea, idc.get_func_name(ea)))
    try:
        out.write(str(ida_hexrays.decompile(ea)) + "\n")
    except Exception as e:
        out.write("ERR %s\n" % e)
out.close()
