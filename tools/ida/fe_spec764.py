# find every instruction with displacement 0x2FC in the FE code range 0x240000..0x2A0000 (missed by gfx_spec_xr)
import idautils, idc, ida_funcs, ida_ua
out = r"D:/projects/project poland rewrite/research/fe_spec764.txt"
with open(out, "w", encoding="utf-8") as f:
    for lo, hi in ((0x200000, 0x2A0000),):
        f.write("== disp 2FC in %X..%X\n" % (lo, hi))
        for fs in idautils.Functions(lo, hi):
            for ea in idautils.FuncItems(fs):
                insn = ida_ua.insn_t()
                if not ida_ua.decode_insn(insn, ea):
                    continue
                for op in insn.ops:
                    if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and (op.addr & 0xFFFFFFFF) == 0x2FC:
                        f.write("  %08X %s %s\n" % (ea, idc.get_func_name(fs), idc.generate_disasm_line(ea, 0)))
                    if op.type == ida_ua.o_imm and op.value == 0x2FC:
                        f.write("  imm %08X %s %s\n" % (ea, idc.get_func_name(fs), idc.generate_disasm_line(ea, 0)))
print("wrote", out)
