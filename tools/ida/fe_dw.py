# frontend agent: dwords listed in research/fe_dw_args.txt -> research/fe_dw.txt
import ida_bytes, struct
out = open(r"D:\projects\project poland rewrite\research\fe_dw.txt", "w")
for a in [int(x,16) for x in open(r"D:\projects\project poland rewrite\research\fe_dw_args.txt").read().split()]:
    v = ida_bytes.get_dword(a)
    out.write("%08X %08X %d %g\n" % (a, v, v - (1 << 32) if v >> 31 else v, struct.unpack("<f", struct.pack("<I", v))[0]))
out.close()
