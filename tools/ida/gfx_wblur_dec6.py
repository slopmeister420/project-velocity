# wheel blur extra pass: 0x114A90 / 0x114C10 and helpers -> research/gfx/wblur_dec6.c
import idc, ida_hexrays, idautils
out = []
for ea in (0x114A90, 0x114C10, 0x138F0, 0xE98D0):
    try:
        out.append("//==== %X %s" % (ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(ea)))
    except Exception as e:
        out.append("// fail %s" % e)
for name in ("byte_7181FC", "dword_686610"):
    ea = idc.get_name_ea_simple(name)
    out.append("// xrefs %s %X: %s" % (name, ea, " ".join("%X(%s)" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(ea))))
out.append("// 686610 = %08X" % idc.get_wide_dword(0x686610))
open(r"D:\projects\project poland rewrite\research\gfx\wblur_dec6.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
