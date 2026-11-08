# call order in GameManager 0xB9010 around the car render -> research/gfx/neon_gm_calls.txt
import idc, idautils, ida_funcs
B = r"D:\projects\project poland rewrite\research\gfx"
out = []
for x in idautils.XrefsTo(0x119900):
    out.append("119900 called at %X in %s" % (x.frm, idc.get_func_name(x.frm)))
f = ida_funcs.get_func(0xB9010)
ea = f.start_ea
while ea < f.end_ea:
    if idc.print_insn_mnem(ea) == "call":
        t = idc.get_operand_value(ea, 0)
        n = idc.get_func_name(t)
        if n and ("CarRender" in n or "TrackFX" in n):
            out.append("%X call %s" % (ea, n))
    ea = idc.next_head(ea)
open(B + r"\neon_gm_calls.txt", "w", encoding="utf-8").write("\n".join(out))
print("ok")
