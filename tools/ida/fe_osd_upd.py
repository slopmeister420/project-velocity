import ida_hexrays, ida_funcs, idaapi, idc, ida_name
out = open(r"D:\projects\project poland rewrite\research\fe_osd_upd.c", "w")
ea = ida_name.get_name_ea(idaapi.BADADDR, "?GetRaceLength@VsAIWorld@@QAEHXZ")
if ea == idaapi.BADADDR:
    for a in idautils_list if False else []:
        pass
import idautils
for a, n in idautils.Names():
    if "GetRaceLength" in n:
        out.write("%x %s\n" % (a, n))
        try:
            out.write(str(ida_hexrays.decompile(a)) + "\n\n")
        except Exception as e:
            out.write("fail %s\n" % e)
out.close()
