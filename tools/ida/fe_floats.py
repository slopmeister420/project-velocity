# frontend agent: dump dwords as hex + float (fe_peek_args.txt: start end)
import idc, struct
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
s, e = int(a[0], 16), int(a[1], 16)
for g in range(s, e, 4):
    v = idc.get_wide_dword(g)
    print("%08X %08X %r %s" % (g, v, struct.unpack("<f", struct.pack("<I", v))[0], idc.get_name(g)))
