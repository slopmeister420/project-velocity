# find small helpers called from the ai regions that return float in st0 and retype them (ai agent)
import idautils, idc, ida_funcs, ida_hexrays, ida_typeinf, ida_nalt, ida_ua

AI_LO, AI_HI = 0x12440, 0x43610
EXTRA = [(0xE72C0, 0xE85A0), (0x16E980, 0x16F5B0)]
DRY = False

NONRET = {"fstp", "fist", "fistp", "fisttp", "fcom", "fcomp", "fcompp", "fucom", "fucomp", "fucompp",
          "fnstsw", "fstsw", "ftst", "fnstcw", "fstcw", "fldcw", "ffree", "fwait", "wait", "fnclex", "fxam", "fninit"}
def is_fpu_load(m):
    return m not in NONRET

FLOATSET = set()
def returns_float(fea):
    f = ida_funcs.get_func(fea)
    if not f or f.end_ea - f.start_ea > 0x1000:
        return False
    last_fpu = None
    after = []
    for h in idautils.Heads(f.start_ea, f.end_ea):
        m = idc.print_insn_mnem(h)
        if m.startswith("f"):
            last_fpu = m
            after = []
        else:
            after.append((m, idc.print_operand(h, 0) if m != "call" else idc.get_operand_value(h, 0)))
    calls = [op for m, op in after if m == "call" and op not in (0x3DDC30, 0x3DDC60)]
    if calls:
        return calls[-1] in FLOATSET or "sqrt" in idc.get_func_name(calls[-1]).lower()
    if last_fpu is None or not is_fpu_load(last_fpu):
        return False
    for m, op in after:
        if m in ("pop", "leave", "retn", "ret", "add", "cmp", "jmp", "nop", "int3"):
            continue
        if m in ("push", "lea"):
            continue
        if m == "mov" and op != "eax":
            continue
        if m == "call":
            continue  # _RTC_CheckEsp
        return False
    return True

callees = set()
ranges = [(AI_LO, AI_HI)] + EXTRA
for lo, hi in ranges:
    for fea in idautils.Functions(lo, hi):
        callees.add(fea)
        f = ida_funcs.get_func(fea)
        for h in idautils.Heads(f.start_ea, f.end_ea):
            if idc.print_insn_mnem(h) == "call":
                t = idc.get_operand_value(h, 0)
                if 0x11000 <= t < 0x3DDD20:
                    callees.add(t)

# fixpoint over helpers that end in a call to a float helper
while True:
    n = len(FLOATSET)
    for fea in callees:
        if fea not in FLOATSET and returns_float(fea):
            FLOATSET.add(fea)
    if len(FLOATSET) == n:
        break
changed = 0
for fea in sorted(FLOATSET):
    tif = ida_typeinf.tinfo_t()
    if not ida_nalt.get_tinfo(tif, fea):
        try:
            cf = ida_hexrays.decompile(fea)
        except Exception:
            continue
        cf.get_func_type(tif)
    fi = ida_typeinf.func_type_data_t()
    if not tif.get_func_details(fi):
        continue
    rt = fi.rettype
    if rt.is_float():
        continue
    fi.rettype = ida_typeinf.tinfo_t(ida_typeinf.BT_FLOAT)
    nt = ida_typeinf.tinfo_t()
    nt.create_func(fi)
    print("%08X %s : %s -> float" % (fea, idc.get_func_name(fea), rt.dstr()))
    if not DRY:
        ida_typeinf.apply_tinfo(fea, nt, ida_typeinf.TINFO_DEFINITE)
    changed += 1
print("changed", changed)
