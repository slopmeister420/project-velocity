import idautils, idc, ida_funcs
out = []
for a in range(0x6319E0, 0x631A30, 4):
    v = idc.get_wide_dword(a)
    out.append("%X %X %s xrefs:%s" % (a, v, idc.get_func_name(v) or "", " ".join("%X@%s" % (x.frm, idc.get_func_name(x.frm)) for x in idautils.XrefsTo(a))))
# who passes 0x6319xx / callbacks in B4170 near Graphics.Camera
open(r"D:/projects/project poland rewrite/research/core_camtweak_vt.txt", "w").write("\n".join(out))
print("ok")
