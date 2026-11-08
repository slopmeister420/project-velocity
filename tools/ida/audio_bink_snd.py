import ida_hexrays, idc, idautils, ida_funcs, ida_bytes
# bink's default xbox sound driver (unk_544280) and the functions it installs -> research/audio/bink_snd.c
ida_hexrays.init_hexrays_plugin()
out = []
segs = []
for s in idautils.Segments():
    segs.append("%s %X-%X" % (idc.get_segm_name(s), s, idc.get_segm_end(s)))
out.append("// segments with 0x544280: " + idc.get_segm_name(0x544280))
out.append("// " + " | ".join(x for x in segs if "BINK" in x.upper() or "DSOUND" in x.upper()))
todo = [0x544280]
seen = set()
while todo and len(seen) < 40:
    ea = todo.pop(0)
    if ea in seen:
        continue
    seen.add(ea)
    if ida_funcs.get_func(ea) is None:
        ida_funcs.add_func(ea)
    f = ida_funcs.get_func(ea)
    if f is None:
        out.append("// ---- %X no func" % ea)
        continue
    err = ida_hexrays.hexrays_failure_t()
    c = None
    try:
        c = ida_hexrays.decompile(ea, err)
    except Exception as e:
        pass
    if c is not None:
        out.append("// ---- %X %s\n%s" % (ea, idc.get_func_name(ea), str(c)))
    else:
        dis = ["%X  %s" % (h, idc.generate_disasm_line(h, 0)) for h in idautils.FuncItems(ea)]
        out.append("// ---- %X %s (asm: %s)\n%s" % (ea, idc.get_func_name(ea), err.desc(), "\n".join(dis)))
    # follow offsets to code (installed function pointers) and direct calls inside the bink/dsound area
    for h in idautils.FuncItems(ea):
        for i in range(2):
            v = idc.get_operand_value(h, i)
            if 0x53E000 <= v < 0x552000 and idc.get_operand_type(h, i) in (5, 2) and not (f.start_ea <= v < f.end_ea):
                todo.append(v)
        for r in idautils.CodeRefsFrom(h, 0):
            if 0x53E000 <= r < 0x552000 and not (f.start_ea <= r < f.end_ea):
                todo.append(r)
open(r"D:\projects\project poland rewrite\research\audio\bink_snd.c", "w").write("\n".join(out))
