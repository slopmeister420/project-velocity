# physics agent: name + type the inline float math wrappers (identified through the FastCar expression
# function table), and retype st0-returning functions in the physics ranges
import idc, ida_typeinf, ida_name

MATH = {
    0x64B90: ("VsFabs", "float __cdecl f(float x);"),
    0x24930: ("VsAcos", "float __cdecl f(float x);"),
    0x477B0: ("VsAsin", "float __cdecl f(float x);"),
    0x5EC30: ("VsAtan", "float __cdecl f(float x);"),
    0x246B0: ("VsCos", "float __cdecl f(float x);"),
    0x1621A0: ("VsExp", "float __cdecl f(float x);"),
    0x313B00: ("VsLog10", "float __cdecl f(float x);"),
    0x24580: ("VsSin", "float __cdecl f(float x);"),
    0x5EB00: ("VsTan", "float __cdecl f(float x);"),
    0x130F0: ("VsSqrt", "float __cdecl f(float x);"),
    0x47800: ("VsAtan2", "float __cdecl f(float y, float x);"),
    0x7CF60: ("VsPow", "float __cdecl f(float x, float y);"),
    0x141E0: ("VsClamp", "float __cdecl f(float v, float lo, float hi);"),
}
for ea, (nm, decl) in MATH.items():
    cur = idc.get_func_name(ea)
    if not cur.startswith("Vs") or cur.split("__")[0] != cur or cur == nm:
        pass
    if "__" in cur or cur.startswith("sub_"):
        ida_name.set_name(ea, nm, ida_name.SN_NOWARN | ida_name.SN_FORCE)
    ok = idc.SetType(ea, decl)
    print("%08X %-12s -> %-10s type %s" % (ea, cur, idc.get_func_name(ea), ok))

# own-range functions that return a float in st0
FLOATRET = [0x78090, 0x1B8980, 0x1DA650, 0x35CBE0, 0x35DB90, 0x364C70, 0x364D20, 0x378F60, 0x3AB310]
for ea in FLOATRET:
    tif = ida_typeinf.tinfo_t()
    import ida_nalt
    if ida_nalt.get_tinfo(tif, ea) and tif.is_func():
        fi = ida_typeinf.func_type_data_t()
        tif.get_func_details(fi)
        fl = ida_typeinf.tinfo_t()
        fl.create_simple_type(ida_typeinf.BTF_FLOAT)
        fi.rettype = fl
        # drop a bogus st0 input argument
        keep = ida_typeinf.func_type_data_t()
        nt = ida_typeinf.tinfo_t()
        nt.create_func(fi)
        print("%08X float ret %s" % (ea, ida_typeinf.apply_tinfo(ea, nt, ida_typeinf.TINFO_DEFINITE)))
    else:
        print("%08X no tinfo" % ea)
