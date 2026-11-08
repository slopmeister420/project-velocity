import idc, idautils
# data xrefs (function + r/w) of graphics tweak variables without a known reader -> research/gfx/tweak_xrefs.txt
ADDRS = [0x706574, 0x706540, 0x706580, 0x706554, 0x7065BB, 0x7065BC, 0x6866A0, 0x686664, 0x686670, 0x68664C,
         0x686650, 0x706530, 0x706584, 0x706534, 0x7065BA, 0x686610, 0x686600, 0x7065A8, 0x7065A0, 0x706598,
         0x70659C, 0x706544, 0x709D48, 0x718164, 0x718E79, 0x718E7A, 0x718E7C, 0x686FB0, 0x6872E8, 0x6872EC,
         0x6872F0, 0x73694A, 0x6870BC, 0x7486AC, 0x686E7C, 0x686E70, 0x730DE4, 0x730DE5, 0x736584, 0x687184,
         0x6852BC, 0x6D6359]
o = open(r"D:\projects\project poland rewrite\research\gfx\tweak_xrefs.txt", "w")
for a in ADDRS:
    rows = []
    for x in idautils.DataRefsTo(a):
        f = idc.get_func_attr(x, idc.FUNCATTR_START)
        if f == idc.BADADDR:
            continue
        rows.append("%x:%s" % (f, idc.print_insn_mnem(x)))
    o.write("%x %s\n" % (a, " ".join(sorted(set(rows)))))
o.close()
print("ok")
