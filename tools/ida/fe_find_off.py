# frontend agent: find functions that both reference 0x7806C8 and touch [reg+14h]; print the lines
import idautils, idc, ida_funcs
seen = set()
for x in idautils.XrefsTo(0x7806C8):
    f = ida_funcs.get_func(x.frm)
    if not f or f.start_ea in seen:
        continue
    seen.add(f.start_ea)
    for h in idautils.Heads(f.start_ea, f.end_ea):
        d = idc.GetDisasm(h)
        if "+14h]" in d and idc.print_insn_mnem(h) in ("mov", "fstp", "movss") and idc.get_operand_type(h, 0) == idc.o_displ:
            print("%08X %-30s %s" % (h, idc.get_func_name(h), d))
for a in (0x7806DC,):
    for x in idautils.XrefsTo(a):
        print("direct", hex(x.frm), idc.GetDisasm(x.frm))
