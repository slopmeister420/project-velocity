import idautils, idc, ida_bytes
out = open(r"D:\projects\project poland rewrite\research\fe_career_xref.txt", "w")
ea = 0x42B760
last = None
while ea < 0x42B920:
    m = idc.print_insn_mnem(ea)
    if m == "push" and idc.get_operand_type(ea, 0) == idc.o_imm:
        v = idc.get_operand_value(ea, 0)
        s = idc.get_strlit_contents(v)
        if s:
            last = s
    if m == "mov" and idc.get_operand_type(ea, 0) == idc.o_mem:
        out.write("%08x <- %s\n" % (idc.get_operand_value(ea, 0), last))
    ea = idc.next_head(ea)
out.close()
