# frontend agent: list static colour constants built with VISOS_Colour(a,r,g,b) in the crt init area
import idautils, idc
ctor = idc.get_name_ea_simple("VISOS_ColourXBOX__2B11C0")
for x in idautils.XrefsTo(ctor):
    if x.frm < 0x3DDD20:
        continue
    ea = x.frm
    tgt = None
    pushes = []
    p = ea
    for _ in range(6):
        p = idc.prev_head(p)
        m = idc.print_insn_mnem(p)
        if m == "mov" and idc.print_operand(p, 0) == "ecx":
            tgt = idc.get_operand_value(p, 1)
        elif m == "push":
            pushes.append(idc.get_operand_value(p, 0))
    if tgt:
        print("%08X  %-14s argb(%s)" % (tgt, idc.get_name(tgt), ",".join(str(v) for v in pushes[::-1][::-1])))
