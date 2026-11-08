import idautils, idc, ida_ua
out = open(r"D:\projects\project poland rewrite\research\core_s9_off.txt", "w")
for f in idautils.Functions(0x11000, 0x3DDD20):
    n = idc.get_func_name(f)
    for h in idautils.FuncItems(f):
        insn = ida_ua.insn_t()
        if not ida_ua.decode_insn(insn, h): continue
        op = insn.ops[0]
        if op.type == ida_ua.o_displ and op.addr == 0xB78 and idc.print_insn_mnem(h).startswith(('mov','fst')):
            out.write("%08X %s  %s\n" % (h, n, idc.GetDisasm(h)))
out.close()
