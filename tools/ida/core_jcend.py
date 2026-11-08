import ida_hexrays, ida_funcs, idc
out = []
for ea in (0x0005C8A0,):
    f = ida_funcs.get_func(ea)
    try:
        out.append("// %x %s\n" % (ea, idc.get_func_name(ea)) + str(ida_hexrays.decompile(f)))
    except Exception as e:
        out.append("// %x fail %s" % (ea, e))
out.append("// disasm around 0xB8329")
ea = 0x000B8300
while ea < 0x000B83A0:
    out.append("%x  %s" % (ea, idc.GetDisasm(ea)))
    ea = idc.next_head(ea, 0x000B83A0)
# xrefs to dword_68580C
import idautils
out.append("// xrefs 68580C")
for x in idautils.XrefsTo(0x0068580C):
    out.append("%x %s  %s" % (x.frm, idc.get_func_name(x.frm), idc.GetDisasm(x.frm)))
open(r"D:\projects\project poland rewrite\research\core_jcend.c", "w").write("\n".join(out))
