# find 8.0 / 0.125 float constants (imm or data refs) used in code 0x290000..0x2B8000 (visos renderer / model)
# -> research/gfx/uv8_refs.txt
import idautils, idc, ida_bytes, struct
out = []
want = {0x41000000: "8.0", 0x3E000000: "0.125"}
for ea in idautils.Heads(0x290000, 0x2B8000):
    if not idc.is_code(idc.get_full_flags(ea)):
        continue
    for k in range(3):
        t = idc.get_operand_type(ea, k)
        v = idc.get_operand_value(ea, k)
        if t == idc.o_imm and v in want:
            out.append("%X %s imm %s | %s" % (ea, idc.get_func_name(ea), want[v], idc.generate_disasm_line(ea, 0)))
        elif t == idc.o_mem and v > 0x400000:
            d = ida_bytes.get_dword(v)
            if d in want:
                out.append("%X %s mem %X=%s | %s" % (ea, idc.get_func_name(ea), v, want[d], idc.generate_disasm_line(ea, 0)))
open(r"D:/projects/project poland rewrite/research/gfx/uv8_refs.txt", "w").write("\n".join(out) + "\n")
