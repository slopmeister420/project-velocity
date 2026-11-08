# retype float-typed ecx (this) args of ai functions to int, and drop phantom st0 args (ai agent)
import idautils, idc, ida_funcs, ida_hexrays, ida_typeinf, ida_nalt

RANGES = [(0x12440, 0x43610), (0xE72C0, 0xE85A0), (0x16E980, 0x16F5B0)]
n = 0
for lo, hi in RANGES:
    for fea in idautils.Functions(lo, hi):
        tif = ida_typeinf.tinfo_t()
        if ida_nalt.get_tinfo(tif, fea):
            continue  # user-typed already
        try:
            cf = ida_hexrays.decompile(fea, flags=ida_hexrays.DECOMP_NO_CACHE)
        except Exception:
            continue
        cf.get_func_type(tif)
        fi = ida_typeinf.func_type_data_t()
        if not tif.get_func_details(fi):
            continue
        dirty = False
        keep = ida_typeinf.func_type_data_t()
        for a in fi:
            loc = a.argloc
            if loc.is_reg1() and loc.reg1() == 1 and a.type.is_floating():  # ecx
                a.type = ida_typeinf.tinfo_t(ida_typeinf.BT_INT32)
                dirty = True
        if dirty:
            nt = ida_typeinf.tinfo_t()
            nt.create_func(fi)
            if ida_typeinf.apply_tinfo(fea, nt, ida_typeinf.TINFO_DEFINITE):
                n += 1
print("retyped", n)
