# camera rain: sprite draw 0x2A9D50, screen size, up vector init -> research/gfx/rain_dec4.c
import idc, ida_hexrays, idautils, idaapi, struct
out = []
def dec(ea):
    try:
        f = idaapi.get_func(ea)
        out.append("//==== %X %s" % (f.start_ea, idc.get_func_name(ea)))
        out.append(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as e:
        out.append("// fail %X %s" % (ea, e))
for ea in (0x2A9D50, 0x12E520, 0x47840, 0x2ABE30):
    dec(ea)
out.append("// disasm 431EE0..431F20")
ea = 0x431EE0
while ea < 0x431F20:
    out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
    ea = idc.next_head(ea, 0x431F20)
open(r"D:\projects\project poland rewrite\research\gfx\rain_dec4.c", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
