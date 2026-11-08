# frontend agent: disassemble around static-initializer writes to frontend globals
import idautils, idc, ida_funcs
for tgt in (0x766E90, 0x77FD3C, 0x77FD9C, 0x77FD50, 0x77FD30):
    for x in idautils.XrefsTo(tgt):
        if x.frm < 0x3DDD20:
            continue
        f = ida_funcs.get_func(x.frm)
        print("== target %X from %X func %s" % (tgt, x.frm, hex(f.start_ea) if f else None))
        ea = x.frm
        for _ in range(14):
            ea = idc.prev_head(ea)
        for _ in range(20):
            print("   %08X %s" % (ea, idc.GetDisasm(ea)))
            ea = idc.next_head(ea)
        break
