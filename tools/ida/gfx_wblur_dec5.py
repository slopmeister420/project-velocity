# wheel blur: who creates renderer +0x24 (spoke fade shader) -> research/gfx/wblur_dec5.txt
import idc
out = []
ea = 0x10C000
prev = []
while ea < 0x11C000 and ea != idc.BADADDR:
    d = idc.generate_disasm_line(ea, 0)
    prev.append((ea, d))
    prev = prev[-60:]
    if idc.print_insn_mnem(ea) == "mov" and idc.print_operand(ea, 0).endswith("+24h]") and "CreateShader" in "".join(x[1] for x in prev[-4:]):
        out.append("//==== store at %X in %s" % (ea, idc.get_func_name(ea)))
        for a, l in prev:
            out.append("%X %s" % (a, l))
    ea = idc.next_head(ea, 0x11C000)
open(r"D:\projects\project poland rewrite\research\gfx\wblur_dec5.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", len(out))
