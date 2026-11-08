import idc, idautils, ida_hexrays
base = r"D:\projects\project poland rewrite\research"
v = idc.get_wide_dword(0x6C18D8)
out = ["off_6C18D8 = %X %s" % (v, idc.get_func_name(v))]
for x in list(idautils.XrefsTo(0x6C18D8))[:4000]:
    d = idc.generate_disasm_line(x.frm, 0)
    if d.startswith("mov     ds:") or "offset" in d and d.startswith("mov     off_6C18D8"):
        out.append("write %X %s %s" % (x.frm, idc.get_func_name(x.frm), d))
try:
    out.append(str(ida_hexrays.decompile(v)))
except Exception as e:
    out.append("decomp fail %s" % e)
# ProjectOntoLine body
try:
    out.append(str(ida_hexrays.decompile(0x27E00)))
except Exception as e:
    out.append("fail %s" % e)
open(base + r"\ai_assert_handler.txt", "w").write("\n".join(out))
