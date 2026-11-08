# frontend agent: for each CLayer::CreateElement call in [start,end), print the ecx source and the element name crc string
import idautils, idc, ida_funcs
target = idc.get_name_ea_simple("FECK2::CLayer::CreateElement")
args = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
s, e = int(args[0], 16), int(args[1], 16)
for x in idautils.XrefsTo(target):
    if not (s <= x.frm < e):
        continue
    ea = x.frm
    ecx = None
    name = None
    p = ea
    for _ in range(60):
        p = idc.prev_head(p)
        d = idc.GetDisasm(p)
        if ecx is None and idc.print_insn_mnem(p) == "mov" and idc.print_operand(p, 0) == "ecx":
            ecx = d
        if name is None and "CCRCChecked" in d:
            q = p
            for _ in range(4):
                q = idc.prev_head(q)
                if "offset" in idc.GetDisasm(q) and ";" in idc.GetDisasm(q):
                    name = idc.GetDisasm(q).split(";", 1)[1].strip()
                    break
    store = None
    q = ea
    for _ in range(4):
        q = idc.next_head(q)
        if idc.print_insn_mnem(q) == "mov" and idc.print_operand(q, 1) == "eax":
            store = idc.GetDisasm(q)
            break
    print("%08X %-28s ecx<- %-30s store %s  name %s" % (ea, idc.get_func_name(ea), ecx, store, name))
