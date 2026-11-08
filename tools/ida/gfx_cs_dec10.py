import ida_hexrays, idautils, idc
B = r"D:\projects\project poland rewrite\research\gfx"
out = [str(ida_hexrays.decompile(0x1587D0))]
for x in idautils.XrefsTo(0x1587D0):
    out.append("// 1587D0 caller %X in %s" % (x.frm, idc.get_func_name(x.frm)))
open(B + r"\cs_dec10.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
