# create the undefined functions in 0x113AD3..0x114A90 (prologue push ebp / mov ebp,esp), list their xrefs, decompile
import idautils, idc, ida_funcs, ida_bytes, ida_hexrays
out = "D:/projects/project poland rewrite/research/gfx/ptset_mk.txt"
starts = []
ea = 0x113AD3
while ea < 0x114A90:
    if ida_bytes.get_byte(ea) == 0x55 and ida_bytes.get_word(ea + 1) == 0xEC8B and not ida_funcs.get_func(ea):
        ida_funcs.add_func(ea)
        starts.append(ea)
    ea += 1
with open(out, "w", encoding="utf-8") as f:
    for s in starts:
        fn = ida_funcs.get_func(s)
        f.write("//==== %08X..%08X\n" % (s, fn.end_ea if fn else 0))
        for x in idautils.XrefsTo(s):
            f.write("// xref %08X %s type %d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
        try:
            f.write(str(ida_hexrays.decompile(s)) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok", [hex(s) for s in starts])
