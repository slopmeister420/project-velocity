import idautils, idc, ida_ua
out = []
for ea in idautils.Heads(0xB0000, 0xC5000):
    if not idc.is_code(idc.get_full_flags(ea)):
        continue
    if idc.print_insn_mnem(ea) not in ("mov", "or"):
        continue
    if idc.get_operand_type(ea, 0) != idc.o_displ or idc.get_operand_value(ea, 0) != 0x18:
        continue
    if idc.get_operand_type(ea, 1) != idc.o_imm:
        continue
    v = idc.get_operand_value(ea, 1) & 0xFFFFFFFF
    if v > 0xFF:
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
open(r"D:/projects/project poland rewrite/research/core_viewflags.txt", "w").write("\n".join(out))
print(len(out))
