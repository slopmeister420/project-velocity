import idc, idautils, ida_funcs, ida_bytes, ida_nalt
out = open(r"D:\projects\project poland rewrite\research\fe_cov_names.tsv", "w")
def sget(ea):
    if not (0x11000 <= ea < 0x800000): return None
    t = ida_nalt.get_str_type(ea)
    if t is None or t < 0: return None
    try:
        b = ida_bytes.get_strlit_contents(ea, -1, t)
    except Exception:
        return None
    if not b or len(b) < 3: return None
    try: return b.decode("latin1")[:60].replace("\t", " ").replace("\n", " ").replace("\r", " ")
    except Exception: return None
for l in open(r"D:\projects\project poland rewrite\research\fe_cov_raw.tsv"):
    a = int(l.split("\t")[0], 16)
    n = idc.get_func_name(a)
    f = ida_funcs.get_func(a)
    nc = len(list(idautils.CodeRefsTo(a, 0)))
    strs = []; ncall = 0; src = ""
    if f:
        for h in idautils.FuncItems(a):
            m = idc.print_insn_mnem(h)
            if m == "call": ncall += 1
            for i in range(2):
                if idc.get_operand_type(h, i) == idc.o_imm:
                    s = sget(idc.get_operand_value(h, i))
                    if s:
                        if ".cpp" in s or ".h" in s[-3:]: src = s
                        elif len(strs) < 3: strs.append(s)
    out.write("%08X\t%s\t%d\t%d\t%s\t%s\n" % (a, n, nc, ncall, src, "|".join(strs)))
out.close()
