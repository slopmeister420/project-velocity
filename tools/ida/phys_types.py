# physics agent: declare the CConfig structs and type the ::GC global
import idc, ida_name
src = open(r"D:\projects\project poland rewrite\research\phys_dump\cconfig_types.h").read()
print("parse_decls errors:", idc.parse_decls(src, idc.PT_SILENT))
ida_name.set_name(0x74B618, "GC", ida_name.SN_NOWARN | ida_name.SN_FORCE)
print("GC type", idc.SetType(0x74B618, "CGlobalConfig *GC;"))
