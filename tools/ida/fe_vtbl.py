# frontend agent: for each ctor in fe_peek_args.txt find vtable stores and list entries
import idautils, idc, ida_funcs, ida_bytes
args = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
def dump_ctor(ea, depth=0):
    f = ida_funcs.get_func(ea)
    if not f:
        return
    for h in idautils.Heads(f.start_ea, f.end_ea):
        m = idc.print_insn_mnem(h)
        if m == "call" and depth < 1:
            t = idc.get_operand_value(h, 0)
            print("  " * depth + "  call %08X %s" % (t, idc.get_func_name(t)))
        if m == "mov" and idc.get_operand_type(h, 1) == idc.o_imm:
            v = idc.get_operand_value(h, 1)
            if 0x600000 <= v < 0x700000 and idc.get_operand_type(h, 0) in (idc.o_phrase, idc.o_displ) and idc.print_operand(h, 0).startswith("dword ptr [e") and "+" not in idc.print_operand(h, 0):
                print("  " * depth + "  vtbl %08X" % v)
                for i in range(40):
                    p = ida_bytes.get_dword(v + 4 * i)
                    if not (0x11000 <= p < 0x400000):
                        break
                    print("     [%2d] +%02X %08X %s" % (i, 4 * i, p, idc.get_func_name(p)))
for a in args:
    ea = int(a, 16)
    print("=" * 10, hex(ea), idc.get_func_name(ea))
    dump_ctor(ea)
