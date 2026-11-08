import idautils, idc
out = []
for f in idautils.Functions(0x11000, 0x3E000):
    for ea in idautils.FuncItems(f):
        if idc.print_insn_mnem(ea) == "mov" and idc.get_operand_type(ea, 0) in (3, 4):
            t = idc.print_operand(ea, 0)
            if "+7Ch]" in t:
                out.append("%X %s %s" % (ea, idc.get_func_name(ea), idc.generate_disasm_line(ea, 0)))
open(r"D:\projects\project poland rewrite\research\ai_scan7c.txt", "w").write("\n".join(out))
