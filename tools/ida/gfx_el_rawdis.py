# raw disassembly of the ranges in research/gfx/el_raw_in.txt ("start end" hex) -> el_raw_out.txt
import idc, ida_bytes, ida_ua, struct
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for l in open(B + r"\el_raw_in.txt"):
    p = l.split("#")[0].split()
    if len(p) < 2:
        continue
    ea, end = int(p[0], 16), int(p[1], 16)
    out.append("==== %X..%X" % (ea, end))
    while ea < end:
        n = ida_ua.create_insn(ea)
        if n <= 0:
            ea += 1
            continue
        s = idc.GetDisasm(ea)
        extra = ""
        for i in range(2):
            if idc.get_operand_type(ea, i) == idc.o_mem:
                v = idc.get_operand_value(ea, i)
                d = ida_bytes.get_dword(v)
                extra += "  ; [%X]=%r" % (v, struct.unpack("<f", struct.pack("<I", d))[0])
            if idc.get_operand_type(ea, i) == idc.o_imm:
                v = idc.get_operand_value(ea, i) & 0xFFFFFFFF
                if v > 0x10000:
                    extra += "  ; imm=%r" % struct.unpack("<f", struct.pack("<I", v))[0]
        out.append("%06X %s%s" % (ea, s, extra))
        ea += n
open(B + r"\el_raw_out.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
