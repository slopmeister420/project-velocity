# where are the char element's anim names (+768 = 0x300, 7 x 256) and +764 written? list funcs using displ 0x300 / 0x2FC across game code
import idautils, idc, ida_ua
out = r"D:/projects/project poland rewrite/research/fe_spec764e.txt"
res = {}
for fs in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(fs):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, ea):
            continue
        for op in insn.ops:
            if op.type == ida_ua.o_displ and (op.addr & 0xFFFFFFFF) in (0x300, 0x2FC):
                res.setdefault(fs, []).append(ea)
with open(out, "w", encoding="utf-8") as f:
    for fs in sorted(res):
        if 0x1F0000 <= fs < 0x2B0000:
            f.write("%08X %s\n" % (fs, idc.get_func_name(fs)))
            for ea in res[fs][:4]:
                f.write("    %08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
print("wrote", out)
