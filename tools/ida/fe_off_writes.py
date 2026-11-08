# frontend agent: instructions in the FE code range using the displacements in fe_off_args.txt -> research/fe_off_writes.txt
import idautils, idc, ida_ua
offs = [int(t, 16) for t in open(r"D:\projects\project poland rewrite\tools\ida\fe_off_args.txt").read().split()]
out = open(r"D:\projects\project poland rewrite\research\fe_off_writes.txt", "w")
for f in idautils.Functions(0x1FD000, 0x29F000):
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h):
            continue
        for op in insn.ops:
            if op.type in (ida_ua.o_displ, ida_ua.o_phrase) and op.addr in offs:
                out.write("%08X %s  %s\n" % (h, idc.get_func_name(h), idc.GetDisasm(h)))
out.close()
