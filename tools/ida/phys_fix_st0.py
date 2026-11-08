# physics agent: find functions that return a float in st0 (fld right before the epilogue) but whose
# prototype returns an integer/void, and retype their return to float. Scope: callees of the physics
# regions and the FastCar library, plus the functions in those ranges.
import idautils, idc, ida_funcs, ida_typeinf, ida_nalt, ida_ua, ida_hexrays, ida_bytes

ROOT = r"D:\projects\project poland rewrite\research"
RANGES = [(0x182530, 0x199DC0), (0x199DC0, 0x1E6130), (0x6AFA0, 0x79C90), (0x355B60, 0x3C63D0)]
DRY = False
try:
    DRY = open(ROOT + r"\phys_fix_dry.txt").read().strip() == "1"
except Exception:
    pass

def in_ranges(ea):
    return any(a <= ea < b for a, b in RANGES)

cands = set()
for a, b in RANGES:
    for f in idautils.Functions(a, b):
        cands.add(f)
        for h in idautils.FuncItems(f):
            for x in idautils.CodeRefsFrom(h, 0):
                g = ida_funcs.get_func(x)
                if g and g.start_ea == x:
                    cands.add(x)

EPI = ("pop", "mov", "leave", "add", "retn", "cmp", "call", "nop")

def returns_st0(f):
    fn = ida_funcs.get_func(f)
    if not fn or fn.size() > 0x400:
        return False
    rets = [h for h in idautils.FuncItems(f) if idc.print_insn_mnem(h) == "retn"]
    if not rets:
        return False
    for r in rets:
        # walk back over the epilogue to the instruction that produced the value
        h = r
        ok = False
        for _ in range(10):
            h = idc.prev_head(h, fn.start_ea)
            if h == idc.BADADDR or h < fn.start_ea:
                break
            m = idc.print_insn_mnem(h)
            if m.startswith("fld") or m in ("fild", "fst", "fchs", "fabs", "fsqrt", "fsin", "fcos") or                     (m[:4] in ("fmul", "fadd", "fsub", "fdiv") or m[:5] in ("fimul", "fiadd", "fisub", "fidiv")):
                ok = True
                break
            if m == "call" and "3DDC30" not in idc.print_operand(h, 0) and "RTC" not in idc.print_operand(h, 0):
                # value returned straight from a float callee
                tgt = idc.get_operand_value(h, 0)
                tt = ida_typeinf.tinfo_t()
                if ida_nalt.get_tinfo(tt, tgt) and tt.is_func() and tt.get_rettype().is_floating():
                    ok = True
                break
            if m not in EPI:
                break
            if m == "mov" and idc.print_operand(h, 0) in ("eax", "al", "ax"):
                break
        if not ok:
            return False
    return True

changed = []
for _pass in range(4):
  for f in sorted(cands):
      tif = ida_typeinf.tinfo_t()
      if f >= 0x3DDD20:
          continue
      if not ida_nalt.get_tinfo(tif, f):
          try:
              cf = ida_hexrays.decompile(f)
              if cf is not None:
                  tif = cf.type
          except Exception:
              pass
      rt = None
      if tif.is_func():
          rt = tif.get_rettype()
          if rt.is_floating():
              continue
      if not returns_st0(f):
          continue
      decl = idc.get_type(f) or ""
      changed.append((f, idc.get_func_name(f), decl))
      if DRY:
          continue
      if tif.is_func():
          fi = ida_typeinf.func_type_data_t()
          tif.get_func_details(fi)
          fl = ida_typeinf.tinfo_t()
          fl.create_simple_type(ida_typeinf.BTF_FLOAT)
          fi.rettype = fl
          nt = ida_typeinf.tinfo_t()
          nt.create_func(fi)
          ida_typeinf.apply_tinfo(f, nt, ida_typeinf.TINFO_DEFINITE)
for f, n, d in changed:
    print("%08X %-40s %s" % (f, n, d))
print("count", len(changed))
