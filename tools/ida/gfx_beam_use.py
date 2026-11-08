# beam lights use: refs to the beam object, override caller, global input setters, lookat
import idautils, idc, ida_funcs, ida_hexrays
out = r"D:/projects/project poland rewrite/research/gfx/beam_use.c"
seen = set()
def dc(f, ea):
    fn = ida_funcs.get_func(ea)
    if not fn or fn.start_ea in seen:
        return
    seen.add(fn.start_ea)
    f.write("\n//==== %08X %s\n" % (fn.start_ea, idc.get_func_name(fn.start_ea)))
    try:
        f.write(str(ida_hexrays.decompile(fn.start_ea)) + "\n")
    except Exception as ex:
        f.write("// fail %s\n" % ex)
with open(out, "w", encoding="utf-8") as f:
    for a in range(0x7486B0, 0x7486B0 + 72):
        for x in idautils.XrefsTo(a):
            f.write("// ref %08X -> %08X in %s\n" % (x.frm, a, idc.get_func_name(x.frm)))
    for t in (0x8AC20, 0x106A40, 0x12D090, 0x12D130, 0x15D070):
        for x in idautils.XrefsTo(t):
            f.write("// call %08X -> %08X in %s : %s | %s\n" % (x.frm, t, idc.get_func_name(x.frm),
                    idc.generate_disasm_line(idc.prev_head(x.frm), 0), idc.generate_disasm_line(idc.prev_head(idc.prev_head(x.frm)), 0)))
    for a in range(0x7486B0, 0x7486B0 + 72):
        for x in idautils.XrefsTo(a):
            if x.frm < 0x3DDD20:
                dc(f, x.frm)
    for x in idautils.XrefsTo(0x8AC20):
        dc(f, x.frm)
    f.write("\n// dis 48A0F9\n")
    ea = 0x48A0F9
    for i in range(60):
        f.write("%08X %s\n" % (ea, idc.generate_disasm_line(ea, 0)))
        ea = idc.next_head(ea, ea + 64)
print("wrote", out)
