import idc, idautils, ida_funcs
# call graph of BINK code: func size, line count, callees (bink-local) and data refs to BINKDATA
base = r"D:\projects\project poland rewrite\research"
out = []
for s in idautils.Segments():
    name = idc.get_segm_name(s)
    if name.upper() in ("BINK", "BINKY16", "BINK16", "BINK32"):
        for f in idautils.Functions(s, idc.get_segm_end(s)):
            end = idc.get_func_attr(f, idc.FUNCATTR_END)
            callees = set()
            data = set()
            for h in idautils.FuncItems(f):
                for r in idautils.CodeRefsFrom(h, 0):
                    if idc.get_segm_name(r).upper().startswith("BINK") and (r < f or r >= end):
                        callees.add("%X" % r)
                for r in idautils.DataRefsFrom(h):
                    sn = idc.get_segm_name(r)
                    if sn.upper().startswith("BINKD") or sn.upper().startswith("BINKB"):
                        data.add("%X" % r)
            out.append("%X size=%X callees=%s data=%s" % (f, end - f, ",".join(sorted(callees)), ",".join(sorted(data))))
open(base + r"\fe_bink_cg.txt", "w").write("\n".join(out))
