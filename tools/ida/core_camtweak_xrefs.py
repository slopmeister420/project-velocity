import idautils, idc, ida_funcs
out = []
addrs = [0x6850A0, 0x6850A4, 0x6850A8, 0x685808, 0x6851F0, 0x6851F4, 0x6D5E38]
# every follow-camera global 0x68511C..0x685220 too
for a in range(0x68511C, 0x685230, 4):
    if a not in addrs:
        addrs.append(a)
addrs.append(0x68521D)
for a in addrs:
    refs = []
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        fs = f.start_ea if f else 0
        refs.append("%X@%X%s" % (x.frm, fs, "w" if x.type == 2 else ""))
    out.append("%X\t%s" % (a, " ".join(refs)))
open(r"D:/projects/project poland rewrite/research/core_camtweak_xrefs.txt", "w").write("\n".join(out))
print(len(out))
