# frontend agent: OSDManager functions writing this+0x24 (screen mask) and their callers -> research/fe_osd_mask.txt
import idautils, idc, ida_funcs
out = open(r"D:\projects\project poland rewrite\research\fe_osd_mask.txt", "w")
seen = set()
for ea in idautils.Heads(0xED5A0, 0xF3F60):
    if idc.print_insn_mnem(ea) in ("mov", "or", "and") and idc.get_operand_type(ea, 0) in (3, 4):
        if idc.get_operand_value(ea, 0) == 0x24:
            f = ida_funcs.get_func(ea)
            if f and f.start_ea not in seen:
                seen.add(f.start_ea)
                out.write("## %08X %s  (%08X %s)\n" % (f.start_ea, idc.get_func_name(f.start_ea), ea, idc.GetDisasm(ea)))
                for x in idautils.CodeRefsTo(f.start_ea, 0):
                    p = x; l = []
                    for _ in range(6):
                        p = idc.prev_head(p); l.append("      %08X  %s" % (p, idc.GetDisasm(p)))
                    out.write("  <- %08X %s\n" % (x, idc.get_func_name(x)))
                    out.write("\n".join(reversed(l)) + "\n")
out.close()
