# find raw dword references to 0x113B00 / 0x113B70 / 0x114680 anywhere, and retry the decompile
import idautils, idc, ida_bytes, ida_hexrays, ida_segment, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/ptset_mk2.txt"
targets = [0x113B00, 0x113B70, 0x114680]
with open(out, "w", encoding="utf-8") as f:
    for s in idautils.Segments():
        seg = ida_segment.getseg(s)
        ea = seg.start_ea
        while ea + 4 <= seg.end_ea:
            v = ida_bytes.get_dword(ea)
            if v in targets:
                f.write("raw %08X -> %08X in %s (%s)\n" % (ea, v, ida_segment.get_segm_name(seg), idc.get_func_name(ea)))
            ea += 1 if ea % 4 else 4
    ida_hexrays.init_hexrays_plugin()
    for t in targets:
        f.write("//==== %08X\n" % t)
        try:
            c = ida_hexrays.decompile(t)
            f.write(str(c) + "\n")
        except Exception as ex:
            f.write("// fail %s\n" % ex)
print("ok")
