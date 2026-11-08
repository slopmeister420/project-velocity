import idc, ida_bytes, ida_ua
lo, hi = 0x424540, 0x4246B0
out = []
ea = lo
while ea < hi:
    n = ida_ua.decode_insn(ida_ua.insn_t(), ea)
    if n <= 0:
        ea += 1
        continue
    line = idc.generate_disasm_line(ea, 0)
    # resolve float constants loaded with movss xmm, ds:x
    for op in (0, 1):
        if idc.get_operand_type(ea, op) == idc.o_mem:
            a = idc.get_operand_value(ea, op)
            if not (0x706000 <= a < 0x707000):
                line += '   ; [%X]=%r/%08X' % (a, idc.get_wide_dword(a) and __import__('struct').unpack('<f', ida_bytes.get_bytes(a, 4))[0], idc.get_wide_dword(a))
    out.append('%X %s' % (ea, line))
    ea += n
open(r'D:/projects/project poland rewrite/research/gfx2/paintinit.txt', 'w').write('\n'.join(out))
