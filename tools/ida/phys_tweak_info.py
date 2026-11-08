import idc, idautils, ida_bytes, ida_funcs
# physics tweak globals: initial dword, readers / writers -> research/phys_tweak_info.txt
out = open(r"D:\projects\project poland rewrite\research\phys_tweak_info.txt", "w", encoding="utf-8")
for n in ["a333", "a333Fff", "aFff_0"]:
    out.write("name %s = %X\n" % (n, idc.get_name_ea_simple(n)))
ADDRS = [0x6889F8, 0x6889FC, idc.get_name_ea_simple("a333"), 0x688A04, 0x688A08, 0x688A0C, 0x75B285, 0x75B286,
         0x688A24, 0x688A28, 0x75B287, 0x688A2C, idc.get_name_ea_simple("a333Fff"), 0x688A34, 0x685F68, 0x688A38,
         0x75B288, 0x75B28C, 0x688A74, 0x688A78, 0x688A7C, 0x688A6C, 0x688A68, 0x688A70, 0x688B90, 0x688A80,
         0x688A84, idc.get_name_ea_simple("aFff_0"), 0x688A8C, 0x688A90, 0x688A94, 0x688A98, 0x688A10, 0x688A14,
         0x688A18, 0x688A1C, 0x688A9C, 0x688B80, 0x75B280]
for a in ADDRS:
    out.write("== %X init=%08X loaded=%d\n" % (a, ida_bytes.get_dword(a), ida_bytes.is_loaded(a)))
    for x in idautils.XrefsTo(a):
        if 0x1B8A00 <= x.frm < 0x1B9600:
            continue
        out.write("  %X %s t%d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
out.write("== mat table 0x6879C0\n")
for i in range(26):
    b = 0x6879C0 + 12 * i
    out.write("%d %08X %08X %08X\n" % (i, ida_bytes.get_dword(b), ida_bytes.get_dword(b + 4), ida_bytes.get_dword(b + 8)))
for f in [0x1DC650, 0x19BEC0]:
    out.write("== callers %X\n" % f)
    for x in idautils.XrefsTo(f):
        out.write("  %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
# float step immediates: push args of each tweak__85A90 call in 0x1B8A00 / 0x19BEC0 / 0x1DC650
for f in [0x19BEC0, 0x1B8A00, 0x1DC650]:
    out.write("== pushes %X\n" % f)
    fn = ida_funcs.get_func(f)
    ea = fn.start_ea
    pend = []
    while ea < fn.end_ea:
        m = idc.print_insn_mnem(ea)
        if m == "push":
            pend.append(idc.print_operand(ea, 0))
        if m == "call":
            t = idc.get_operand_value(ea, 0)
            if t in (0x85A90, 0x85B40, 0x85CA0, 0x86250):
                out.write("  %X call %X %s\n" % (ea, t, " ".join(pend[-6:])))
            pend = []
        ea = idc.next_head(ea, fn.end_ea)
out.close()
print("ok")
