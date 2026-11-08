# physics agent stage 4b: callers of physics-object manager / tracker / axis list entry points
import idaapi, idautils, idc
out = open(r"D:\projects\project poland rewrite\research\phys_s4b_xrefs.txt", "w")
T = [0x1DE730, 0x1DFD10, 0x1DFF60, 0x1E0710, 0x1E0EE0, 0x1E1F80, 0x1E18E0, 0x1E1A20, 0x1E1C00, 0x1E2760,
     0x1E4C30, 0x1E4AE0, 0x1E5270, 0x1E4710, 0x1E3CB0, 0x24F90, 0x385920, 0x385B80, 0x1A89F0, 0x1AAED0,
     0x1E58C0, 0x1E5CA0, 0x1E5D40, 0x1E5E80, 0x101320, 0xFFC30, 0x1000E0, 0x1E1D30, 0x1DEBB0]
for t in T:
    out.write("== %X %s\n" % (t, idc.get_func_name(t)))
    for x in idautils.XrefsTo(t):
        f = idaapi.get_func(x.frm)
        out.write("  %X in %X %s\n" % (x.frm, f.start_ea if f else 0, idc.get_func_name(x.frm)))
out.close()
