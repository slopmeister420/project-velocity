# frontend agent: disassemble ranges listed in research/fe_dis_args.txt (start end) -> research/fe_dis.txt
import idc, ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_dis.txt", "w")
a = open(r"D:\projects\project poland rewrite\research\fe_dis_args.txt").read().split()
for i in range(0, len(a), 2):
    ea, end = int(a[i], 16), int(a[i+1], 16)
    while ea < end:
        d = idc.GetDisasm(ea)
        extra = ""
        for op in range(2):
            if idc.get_operand_type(ea, op) == idc.o_mem:
                v = idc.get_operand_value(ea, op)
                x = ida_bytes.get_dword(v)
                extra += "  [%08X]=%08X(%g)" % (v, x, struct.unpack("<f", struct.pack("<I", x))[0])
        out.write("%08X %s%s\n" % (ea, d, extra))
        ea = idc.next_head(ea)
out.close()
