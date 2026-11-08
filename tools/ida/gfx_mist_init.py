# static init of the puddle mist globals -> research/gfx/mist_init.txt
import idc, ida_bytes, ida_ua, struct
lo, hi = 0x4259C0, 0x425BB0
out = []
ea = lo
while ea < hi:
    n = ida_ua.decode_insn(ida_ua.insn_t(), ea)
    if n <= 0:
        ea += 1
        continue
    line = idc.generate_disasm_line(ea, 0)
    for op in (0, 1):
        t = idc.get_operand_type(ea, op)
        if t == idc.o_imm:
            v = idc.get_operand_value(ea, op) & 0xFFFFFFFF
            line += '   ; imm %r' % struct.unpack('<f', struct.pack('<I', v))[0]
    out.append('%X %s' % (ea, line))
    ea += n
open(r'D:/projects/project poland rewrite/research/gfx/mist_init.txt', 'w').write('\n'.join(out))
print('ok')
