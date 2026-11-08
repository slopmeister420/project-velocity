import ida_hexrays, idc, idautils
# decompile every function in the BINK* code sections into research/fe_bink.c
base = r"D:\projects\project poland rewrite\research"
dec = []
for s in idautils.Segments():
    name = idc.get_segm_name(s)
    if name.upper() in ("BINK", "BINKY16", "BINK16", "BINK32"):
        for f in idautils.Functions(s, idc.get_segm_end(s)):
            try:
                dec.append("// ---- %X %s\n%s" % (f, idc.get_func_name(f), str(ida_hexrays.decompile(f))))
            except Exception as e:
                dec.append("// ---- %X failed %s" % (f, e))
open(base + r"\fe_bink.c", "w").write("\n".join(dec))
# imported/xref names called from bink code outside the section
ext = set()
for s in idautils.Segments():
    if idc.get_segm_name(s).upper().startswith("BINK") and idc.get_segm_name(s).upper() not in ("BINKDATA", "BINKBSS"):
        for f in idautils.Functions(s, idc.get_segm_end(s)):
            for h in idautils.FuncItems(f):
                for r in idautils.CodeRefsFrom(h, 0):
                    sn = idc.get_segm_name(r)
                    if not sn.upper().startswith("BINK"):
                        ext.add("%X %X %s" % (f, r, idc.get_func_name(r)))
open(base + r"\fe_bink_ext.txt", "w").write("\n".join(sorted(ext)))
