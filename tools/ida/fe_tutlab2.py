# frontend agent: static init of dword_76A518 (tip line points) -> research/fe_tutlab2.txt
import idc, ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_tutlab2.txt", "w", encoding="utf-8")
ea = 0x431060
while ea < 0x431330:
    line = idc.generate_disasm_line(ea, 0)
    extra = ""
    for n in range(2):
        if idc.get_operand_type(ea, n) == idc.o_imm:
            v = idc.get_operand_value(ea, n) & 0xFFFFFFFF
            extra += " ; %08X=%r" % (v, struct.unpack("<f", struct.pack("<I", v))[0])
    out.write("%08X %s%s\n" % (ea, line, extra))
    ea = idc.next_head(ea, 0x431400)
out.close()
