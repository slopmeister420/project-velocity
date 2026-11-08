import idautils, idc, ida_funcs
out = open(r"D:/projects/project poland rewrite/research/gfx/perf3_order.txt", "w")
f = ida_funcs.get_func(0xB9010)
for ea in idautils.FuncItems(f.start_ea):
    if idc.print_insn_mnem(ea) == "call":
        t = idc.get_operand_value(ea, 0)
        out.write("%X -> %X %s\n" % (ea, t, idc.get_func_name(t)))
    elif idc.print_insn_mnem(ea) in ("test", "and") and "dword_685E88" in idc.GetDisasm(ea) or "flags" in idc.GetDisasm(ea):
        out.write("%X    %s\n" % (ea, idc.GetDisasm(ea)))
out.close()
