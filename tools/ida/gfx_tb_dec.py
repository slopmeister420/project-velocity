# decompile TrailBlazerRender front half + callers -> research/gfx/tb_dec.c
import ida_hexrays, idc, idautils
B = r"D:\projects\project poland rewrite\research\gfx"
eas = list(idautils.Functions(0x14DF70, 0x14F960)) + [0xC04F0, 0xC0730]
out = []
for a in eas:
    out.append("//==== %08X %s  [%s]" % (a, idc.get_func_name(a), idc.get_type(a)))
    try:
        out.append(str(ida_hexrays.decompile(a)))
    except Exception as ex:
        out.append("// fail %s" % ex)
# static init around 0x426540
out.append("//==== static init 0x426530..0x4265C0")
ea = 0x426530
while ea < 0x4265C8:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x4266000)
open(B + r"\tb_dec.c", "w", encoding="utf-8").write("\n".join(out))
print("ok")
