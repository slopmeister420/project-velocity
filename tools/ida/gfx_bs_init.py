# raw disassembly of the CarDamage bonnet smoke static init 0x421CEF..0x421E3F (+ a little before for the
# function start) -> research/gfx/bs_init.txt, float values of memory/immediate operands annotated
import idc, ida_bytes, ida_ua, ida_funcs, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
f = None
start = 0x421C80
end = 0x421F00
out.append("==== %X..%X (func %s)" % (start, end, idc.get_func_name(start)))
ea = start
while ea < end:
    n = ida_ua.create_insn(ea)
    if n <= 0:
        ea += 1
        continue
    s = idc.GetDisasm(ea)
    extra = ""
    for i in range(3):
        t = idc.get_operand_type(ea, i)
        if t == idc.o_mem:
            v = idc.get_operand_value(ea, i)
            d = ida_bytes.get_dword(v)
            extra += "  ; [%X]=%r/%X" % (v, struct.unpack("<f", struct.pack("<I", d))[0], d)
        if t == idc.o_imm:
            v = idc.get_operand_value(ea, i) & 0xFFFFFFFF
            if v > 0x10000:
                extra += "  ; imm=%r" % struct.unpack("<f", struct.pack("<I", v))[0]
    out.append("%06X %s%s" % (ea, s, extra))
    ea += n
open(B + r"\bs_init.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok", hex(start), hex(end), len(out))
