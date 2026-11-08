# physics agent: drop guessed prototypes that carry a bogus st0 input (left over from callees that were
# not yet typed as returning float), so Hex-Rays re-derives them; ecx-as-float becomes a plain this.
import idautils, idc, ida_typeinf, ida_nalt, ida_hexrays, re

RANGES = [(0x182530, 0x199DC0), (0x199DC0, 0x1E6130), (0x6AFA0, 0x79C90), (0x355B60, 0x3C63D0)]
n_clear = 0
for a, b in RANGES:
    for f in idautils.Functions(a, b):
        t = idc.get_type(f) or ""
        if "@<st0>" in t or "@<efl>" in t:
            ida_nalt.del_tinfo(f)
            n_clear += 1
ida_hexrays.clear_cached_cfuncs()
# second step: let the decompiler guess, then fix "float aN@<ecx>" to a thiscall int
n_this = 0
for a, b in RANGES:
    for f in idautils.Functions(a, b):
        try:
            cf = ida_hexrays.decompile(f)
        except Exception:
            continue
        if cf is None:
            continue
        t = str(cf.type)
        if "@<ecx>" in t and re.search(r"float\s*\w*@<ecx>", t) or "float@<ecx>" in t:
            fi = ida_typeinf.func_type_data_t()
            cf.type.get_func_details(fi)
            for i in range(fi.size()):
                arg = fi[i]
                if arg.argloc.is_reg1() and arg.argloc.reg1() == 1 and arg.type.is_floating():
                    it = ida_typeinf.tinfo_t()
                    it.create_simple_type(ida_typeinf.BT_INT32)
                    arg.type = it
            nt = ida_typeinf.tinfo_t()
            nt.create_func(fi)
            if ida_typeinf.apply_tinfo(f, nt, ida_typeinf.TINFO_DEFINITE):
                n_this += 1
print("cleared", n_clear, "this fixed", n_this)
