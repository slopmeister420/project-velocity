# physics agent: declare a skeleton VehicleEntity and retype VehicleEntity methods' this
import idautils, idc, ida_hexrays, ida_typeinf, ida_nalt, re
decl = """
struct CCarStateHistoryEntry { float f[52]; };
struct CCarStateHistory { CCarStateHistoryEntry *entry; int current; int one; int count; };
struct VehicleEntity {
  char pad_0[240];
  char car[2972];
  char pad_CAC[340];
  CLocalConfig *LC;
  int pad_DE4;
  CCarStateHistory history;
  char pad_DF0[2000];
};
"""
print("parse", idc.parse_decls(decl, idc.PT_SILENT))
n = 0
for f in idautils.Functions(0x1BAEA0, 0x1DC650):
    try:
        cf = ida_hexrays.decompile(f)
    except Exception:
        continue
    if cf is None:
        continue
    txt = str(cf)
    if not re.search(r"\+ (3552|3560)\)", txt):
        continue
    fi = ida_typeinf.func_type_data_t()
    if not cf.type.get_func_details(fi) or fi.size() == 0:
        continue
    a0 = fi[0]
    if not (a0.argloc.is_reg1() and a0.argloc.reg1() == 1):
        continue
    t = ida_typeinf.tinfo_t()
    t.get_named_type(None, "VehicleEntity")
    p = ida_typeinf.tinfo_t()
    p.create_ptr(t)
    a0.type = p
    a0.name = "this"
    nt = ida_typeinf.tinfo_t()
    nt.create_func(fi)
    if ida_typeinf.apply_tinfo(f, nt, ida_typeinf.TINFO_DEFINITE):
        n += 1
print("typed", n)
