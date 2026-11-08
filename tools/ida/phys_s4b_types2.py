# physics agent stage 4b: pass prototypes + list callers of every retyped function to research/phys_addrs.txt
import idc, idautils, ida_funcs
print(idc.SetType(0x1A89F0, "unsigned __int64 __thiscall f(_DWORD *this, _DWORD *a2, int a3, int a4, _DWORD *a5);"))
print(idc.SetType(0x1AAED0, "unsigned __int64 __thiscall f(_DWORD *this, _DWORD *a3, int a4, int a5, _DWORD *a6);"))
R = [0x1E4C30, 0x1DEBB0, 0x1E4710, 0x1E3A60, 0x1E1F50, 0x384C70, 0x1E5270, 0x379660, 0x3847C0, 0x379050, 0x1A89F0,
     0x1AAED0]
out = set(R)
for f in R:
    for x in idautils.CodeRefsTo(f, 0):
        g = ida_funcs.get_func(x)
        if g:
            out.add(g.start_ea)
open(r"D:\projects\project poland rewrite\research\phys_addrs.txt", "w").write("\n".join("%X" % a for a in sorted(out)) + "\n")
print(" ".join("%X" % a for a in sorted(out)))
