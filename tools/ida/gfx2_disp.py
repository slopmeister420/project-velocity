# find instructions in game code using a given displacement (list in DISPS); writes research/g2_disp.txt
import idautils, idc, ida_funcs, ida_ua
DISPS = [0xCCC]
out = open(r"D:\projects\project poland rewrite\research\g2_disp.txt", "w")
for fa in idautils.Functions(0x11000, 0x3DDD20):
    for ea in idautils.FuncItems(fa):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, ea):
            continue
        for op in insn.ops:
            if op.type in (idc.o_displ, idc.o_phrase) and op.addr in DISPS:
                out.write("%08X %s %s\n" % (ea, idc.get_func_name(fa), idc.GetDisasm(ea)))
out.close()
print("done")
