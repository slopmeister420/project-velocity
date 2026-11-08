import idaapi, idautils, idc, ida_hexrays, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\core_rain3.txt", "w")
def w(*a):
    out.write(" ".join(str(x) for x in a) + "\n")
for f in (0x00135D30, 0x002AC0C0, 0x0005C820, 0x00023EB0, 0x00023E00, 0x00023D30, 0x00023D50, 0x00024A40, 0x00015D30):
    try:
        w("== decomp", hex(f), idc.get_func_name(f))
        w(str(ida_hexrays.decompile(f)))
    except Exception as ex:
        w("fail", ex)
# 0x181920 caller context: where is 0x181120 called
for x in idautils.CodeRefsTo(0x00181120, 0):
    w("call 181120 from", hex(x), idc.get_func_name(x))
out.close()
