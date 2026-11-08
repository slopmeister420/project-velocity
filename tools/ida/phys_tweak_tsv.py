import idc, ida_bytes, ida_funcs, ida_ua
# physics AddTweaks calls as pushed (bit-exact immediates) -> research/phys_tweak_regs.tsv:
# fn  call-ea  kind(S/F/I/B)  name  var  a0  a1  a2  init-dword-of-var (bss -> "bss")
K = {0x85A90: "F", 0x85B40: "I", 0x85BF0: "E", 0x85CA0: "B", 0x86250: "S"}
out = open(r"D:\projects\project poland rewrite\research\phys_tweak_regs.tsv", "w", encoding="utf-8")
# tweaked variables whose bytes happen to read as strings ("333?", "fff?", ...)
VARS = {0x688A00, 0x688A30, 0x688A34, 0x688A74, 0x688A78, 0x688A88, 0x688A10, 0x688A18, 0x688A1C}
def val(ea):
    t = idc.get_operand_type(ea, 0)
    if t == ida_ua.o_imm or t == ida_ua.o_mem:
        v = idc.get_operand_value(ea, 0) & 0xFFFFFFFF
        s = idc.get_strlit_contents(v, -1, 0) if 0x600000 <= v < 0x800000 and v not in VARS else None
        if s:
            return '"' + s.decode("latin-1") + '"'
        return "%X" % v
    return idc.print_operand(ea, 0)
for f in [0x19BEC0, 0x1B8A00, 0x1DC650]:
    fn = ida_funcs.get_func(f)
    ea, pend = fn.start_ea, []
    while ea < fn.end_ea:
        m = idc.print_insn_mnem(ea)
        if m == "push":
            pend.append(val(ea))
        elif m == "call":
            t = idc.get_operand_value(ea, 0)
            if t in K:
                k = K[t]
                if k == "S":
                    out.write("%x\t%X\tS\t%s\n" % (f, ea, pend[-1] if pend else "?"))
                else:
                    n = 5 if k in "FI" else 2
                    a = list(reversed(pend[-n:]))
                    name, var = a[0], a[1]
                    try:
                        va = int(var, 16)
                        init = ("%08X" % ida_bytes.get_dword(va)) if ida_bytes.is_loaded(va) else "bss"
                    except ValueError:
                        init = "-"
                    rest = a[2:] + [""] * (3 - len(a[2:]))
                    out.write("%x\t%X\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" % (f, ea, k, name, var, rest[0], rest[1], rest[2], init))
            pend = []
        ea = idc.next_head(ea, fn.end_ea)
out.close()
print("ok")
