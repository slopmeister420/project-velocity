# physics agent: thiscall/float prototypes for CFastCar::Evolve (0x3652D0) and its callees, then
# redecompile them into research/phys_dump/Redump.c (merged; Redump.c wins in the translator)
import os, re, idc, ida_hexrays, ida_funcs, ida_typeinf

PATH = r"D:\projects\project poland rewrite\research\phys_dump\Redump.c"
TYPES = {
    0x3652D0: "void __thiscall f(int this, float h, int keep);",
    0x365250: "void __thiscall f(int this);",
    0x3651F0: "void __thiscall f(int this);",
    0x362A30: "void __thiscall f(int this);",
    0x363810: "void __thiscall f(int this);",
    0x36D740: "void __thiscall f(int this);",
    0x36DA90: "void __thiscall f(float *this, float h, int keep);",
    0x36D9E0: "void __thiscall f(float *this, float h, int keep);",
    0x3724B0: "float __thiscall f(float *this);",
    0x3AEDD0: "float __cdecl f(float a, float b, float c, float d);",
    0x367DB0: "void __cdecl f(float *q, float *w, float h);",
    0x372B70: "void __thiscall f(float *this);",
    0x3E2900: "void __usercall __spoils<eax> f(int size@<eax>);",
}
# _chkstk (sub_3E2900) allocates eax bytes; IDA misses it, which breaks sp analysis
import ida_frame, idautils
for x in idautils.CodeRefsTo(0x3E2900, 0):
    if 0x355B60 <= x < 0x3C63D0:
        prev = idc.prev_head(x)
        if idc.print_insn_mnem(prev) == "mov" and idc.print_operand(prev, 0) == "eax":
            n = idc.get_operand_value(prev, 1)
            nxt = idc.next_head(x)
            idc.add_user_stkpnt(nxt, -n)
            ida_funcs.reanalyze_function(ida_funcs.get_func(x))
            print("chkstk", hex(x), hex(n))
EXTRA = [int(x, 16) for x in os.environ.get("PHYS_EXTRA", "").split()]
have = {}
t = open(PATH, encoding="utf-8").read()
for m in re.finditer(r"(?m)^// ==== ([0-9A-F]{8}) (\S+)\n(.*?)(?=^// ==== |\Z)", t, re.S):
    have[int(m.group(1), 16)] = (m.group(2), m.group(3))
for a, s in TYPES.items():
    tif = ida_typeinf.tinfo_t()
    ida_typeinf.parse_decl(tif, None, s, ida_typeinf.PT_SIL)
    if not ida_typeinf.apply_tinfo(a, tif, ida_typeinf.TINFO_DEFINITE):
        print("type fail", hex(a))
log = []
for a in list(TYPES) + EXTRA:
    try:
        have[a] = (idc.get_func_name(a), str(ida_hexrays.decompile(a, flags=ida_hexrays.DECOMP_NO_CACHE)) + "\n")
    except Exception as ex:
        log.append("fail %x %s" % (a, ex))
with open(PATH, "w", encoding="utf-8") as o:
    for a in sorted(have):
        n, b = have[a]
        o.write("// ==== %08X %s\n%s\n" % (a, n, b.strip("\n")))
print("ok", log)
