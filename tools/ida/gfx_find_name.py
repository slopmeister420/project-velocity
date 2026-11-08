import idautils, idc, ida_hexrays
out=[]
for ea, n in idautils.Names():
    if n in ('VsStringAnsi::Hash','CCRC::CCRCChecked','CCRC::GetCRC'):
        out.append("// %X %s\n%s" % (ea, n, ida_hexrays.decompile(ea)))
open(r"D:/projects/project poland rewrite/research/gfx/q.out","w").write("\n".join(out))
