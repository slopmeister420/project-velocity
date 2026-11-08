# callers (outside the range) of every function in the BoneAnimBlendTree*.cpp range 0x2DB300..0x2DE750
import idc, idautils, ida_funcs
out = "D:/projects/project poland rewrite/research/gfx/s9_bt_xr.txt"
lo, hi = 0x2DB300, 0x2DE750
with open(out, "w", encoding="utf-8") as f:
    for ea in idautils.Functions(lo, hi):
        ext = []
        for x in idautils.XrefsTo(ea):
            fn = ida_funcs.get_func(x.frm)
            if fn is None or not (lo <= fn.start_ea < hi):
                ext.append("%08X %s" % (x.frm, idc.get_func_name(fn.start_ea) if fn else "?(data)"))
        f.write("%08X %s : %s\n" % (ea, idc.get_func_name(ea), "; ".join(ext)))
print("ok")
