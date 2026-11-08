# physics agent: print disassembly lines for ranges in research/phys_lines_args.txt (lines: start end) to phys_dump/lines.txt
import idc, ida_bytes, idaapi
out = []
for line in open(r"D:\projects\project poland rewrite\research\phys_lines_args.txt"):
    p = line.split()
    if len(p) < 2:
        continue
    ea, end = int(p[0], 16), int(p[1], 16)
    while ea < end and ea != idaapi.BADADDR:
        out.append("%08X  %s" % (ea, idc.generate_disasm_line(ea, idc.GENDSM_FORCE_CODE)))
        sz = idc.get_item_size(ea) if idc.is_code(ida_bytes.get_flags(ea)) else (idaapi.decode_insn(idaapi.insn_t(), ea) or 1)
        ea += sz
open(r"D:\projects\project poland rewrite\research\phys_dump\lines.txt", "w").write("\n".join(out))
print(len(out))
