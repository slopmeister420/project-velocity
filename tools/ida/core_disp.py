# instructions in a range whose operand displacement equals one of DISPS -> research/core_disp.txt
import idautils, idc
base = r"D:\projects\project poland rewrite\research"
cfg = open(base + r"\core_disp_in.txt").read().split()
lo, hi = int(cfg[0], 16), int(cfg[1], 16)
disps = {int(x, 16) for x in cfg[2:]}
out = []
for h in idautils.Heads(lo, hi):
    if not idc.is_code(idc.get_full_flags(h)):
        continue
    for i in range(2):
        t = idc.get_operand_type(h, i)
        if t in (3, 4) and (idc.get_operand_value(h, i) & 0xFFFFFFFF) in disps:
            out.append("%X\t%s\t%s" % (h, idc.get_func_name(h), idc.generate_disasm_line(h, 0)))
open(base + r"\core_disp.txt", "w").write("\n".join(out) + "\n")
