import idautils, idc, ida_funcs, ida_ua
# find instructions with given immediates inside the visos audio areas
RANGES = [(0x2b9a00, 0x2bf2d0), (0x3134b0, 0x318520), (0x340400, 0x343290), (0x311d90, 0x3134b0), (0x354f80, 0x356000)]
WANT = {0x69, 22050, 44100, 0x5622, 0x1600, 36, 64, 0x24}
for a, b in RANGES:
    for h in idautils.Heads(a, b):
        if not idc.is_code(idc.get_full_flags(h)):
            continue
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        for op in insn.ops:
            if op.type == ida_ua.o_imm and op.value in WANT:
                f = ida_funcs.get_func(h)
                print(hex(h), idc.get_func_name(f.start_ea) if f else "?", idc.generate_disasm_line(h, 0))
