import ida_segment, ida_funcs, ida_hexrays, idc, idautils, ida_name
# list segments, functions in BINK* sections, and decompile the VsMovie region
base = r"D:\projects\project poland rewrite\research"
out = []
seg = ida_segment.get_first_seg()
while seg:
    out.append("SEG %s %X %X" % (ida_segment.get_segm_name(seg), seg.start_ea, seg.end_ea))
    seg = ida_segment.get_next_seg(seg.start_ea)
for s in idautils.Segments():
    name = idc.get_segm_name(s)
    if "BINK" in name.upper():
        for f in idautils.Functions(s, idc.get_segm_end(s)):
            fn = ida_funcs.get_func(f)
            callers = len(list(idautils.CodeRefsTo(f, 0)))
            out.append("FN %s %X %X %s refs=%d" % (name, f, fn.end_ea, idc.get_func_name(f), callers))
open(base + r"\fe_bink_map.txt", "w").write("\n".join(out))
dec = []
for f in idautils.Functions(0x2F3160, 0x2F6710):
    try:
        dec.append("// ---- %X %s\n%s" % (f, idc.get_func_name(f), str(ida_hexrays.decompile(f))))
    except Exception as e:
        dec.append("// ---- %X failed %s" % (f, e))
open(base + r"\fe_vsmovie.c", "w").write("\n".join(dec))
