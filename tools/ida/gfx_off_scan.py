# graphics agent: instructions in 0x25A000..0x262000 with displacement 0x9AC -> research/gfx/s12_off.txt
import idautils, idc, ida_ua
out = open(r"D:\projects\project poland rewrite\research\gfx\s12_off.txt", "w")
for f in idautils.Functions(0x25A000, 0x262000):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and op.addr in (0x9B4, 0x9B8):
                out.write("%08X %s  %s\n" % (h, idc.get_func_name(h), idc.GetDisasm(h)))
out.close()
