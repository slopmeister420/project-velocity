import idc, idautils
# disassembly of a range to research/core_range.txt (strings resolved)
S, E = 0x19E840, 0x19E8B0
out = []
for ea in idautils.Heads(S, E):
    line = idc.generate_disasm_line(ea, 0)
    for op in range(2):
        v = idc.get_operand_value(ea, op)
        s = idc.get_strlit_contents(v) if v and v > 0x400000 else None
        if s:
            line += "  ; " + s.decode("latin1")
    out.append("%X %s" % (ea, line))
open(r"D:\projects\project poland rewrite\research\core_range.txt", "w").write("\n".join(out))
print("ok")
