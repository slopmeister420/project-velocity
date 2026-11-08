# list displacements 0x1D0..0xA00 used in the char element + FE3DCharacters code, to find a sub-object holding +764
import idautils, idc, ida_funcs, ida_ua
out = r"D:/projects/project poland rewrite/research/fe_spec764b.txt"
hits = {}
with open(out, "w", encoding="utf-8") as f:
    for lo, hi in ((0x25F440, 0x262500), (0x284A30, 0x286300)):
        for fs in idautils.Functions(lo, hi):
            for ea in idautils.FuncItems(fs):
                insn = ida_ua.insn_t()
                if not ida_ua.decode_insn(insn, ea):
                    continue
                for op in insn.ops:
                    if op.type in (ida_ua.o_displ, ida_ua.o_imm):
                        v = (op.addr if op.type == ida_ua.o_displ else op.value) & 0xFFFFFFFF
                        if 0x1D0 <= v <= 0xA00 and v != 0x2FC:
                            hits.setdefault(v, []).append(ea)
    for v in sorted(hits):
        eas = hits[v]
        f.write("%X (%d): %s\n" % (v, v, " ".join("%08X" % e for e in eas[:6])))
        for e in eas[:3]:
            f.write("    %08X %s %s\n" % (e, idc.get_func_name(e), idc.generate_disasm_line(e, 0)))
print("wrote", out)
