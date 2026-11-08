import idc, idautils
# disassembly of the ranges below -> research/core_dbgmenu_dis.txt
R = [(0x000B08A0, 0x000B0A10), (0x00086390, 0x000863E0)]
out = []
for S, E in R:
    out.append("---- %X" % S)
    for ea in idautils.Heads(S, E):
        out.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
open(r"D:\projects\project poland rewrite\research\core_dbgmenu_dis.txt", "w").write("\n".join(out))
print("ok")
