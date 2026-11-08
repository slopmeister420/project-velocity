import idc, ida_hexrays
a = idc.get_name_ea_simple("CSkyRender::DirectionFromAngles")
open(r"D:\projects\project poland rewrite\research\g2_dfa.txt", "w").write("%X\n%s" % (a, ida_hexrays.decompile(a)))
