# frontend agent: print disasm + decompile for addresses listed in fe_peek_args.txt
import idautils, idc, ida_hexrays, ida_funcs
args = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
for a in args:
    ea = int(a, 16)
    f = ida_funcs.get_func(ea)
    print("=" * 20, hex(ea), idc.get_func_name(ea))
    if a.endswith("d"):
        pass
    if f and f.end_ea - f.start_ea < 0x80:
        for h in idautils.Heads(f.start_ea, f.end_ea):
            print("  %08X  %s" % (h, idc.GetDisasm(h)))
    try:
        print(str(ida_hexrays.decompile(ea)))
    except Exception as ex:
        print("fail", ex)
