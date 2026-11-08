# frontend agent: the FELights tweak handler around 0x203E98 (no function defined) + callers of 0x203AE0
import idc, idautils, ida_funcs, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\fe_felights.txt", "w", encoding="utf-8")
for x in idautils.XrefsTo(0x203AE0):
    out.write("203AE0 from %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
for x in idautils.XrefsTo(0x75CA98):
    out.write("75CA98 from %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
ea = 0x203E98
f = ida_funcs.get_func(ea)
out.write("func %s\n" % (hex(f.start_ea) if f else None))
# walk back to a push ebp / mov ebp,esp prologue
s = ea
while s > 0x203B00:
    if idc.get_wide_byte(s) == 0x55 and idc.get_wide_byte(s + 1) == 0x8B and idc.get_wide_byte(s + 2) == 0xEC:
        break
    s -= 1
out.write("prologue %X\n" % s)
if not f:
    ida_funcs.add_func(s)
f = ida_funcs.get_func(ea)
if f:
    out.write("func now %X..%X\n" % (f.start_ea, f.end_ea))
    for x in idautils.XrefsTo(f.start_ea):
        out.write("  from %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
    try:
        out.write(str(ida_hexrays.decompile(f.start_ea)))
    except Exception as e:
        out.write("decomp failed %s\n" % e)
# also the 769E58 xrefs
for x in idautils.XrefsTo(0x769E58):
    out.write("769E58 from %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
out.close()
