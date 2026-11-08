# physics agent stage 4b: decompile VsVector4::VsVector4_XYZ
import idc, ida_hexrays
a = idc.get_name_ea_simple("VsVector4::VsVector4_XYZ")
open(r"D:\projects\project poland rewrite\research\phys_s4b_q.txt", "w").write("%X\n%s" % (a, ida_hexrays.decompile(a)))
