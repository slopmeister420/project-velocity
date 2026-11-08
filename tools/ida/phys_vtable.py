# physics agent: dump vtables (args: research/phys_vt_args.txt lines "addr count")
import idc, ida_bytes
for line in open(r"D:\projects\project poland rewrite\research\phys_vt_args.txt"):
    p = line.split()
    if len(p) < 2:
        continue
    a, n = int(p[0], 16), int(p[1])
    print("vtable %08X" % a)
    for i in range(n):
        f = ida_bytes.get_dword(a + 4 * i)
        print("  [%2d] +%02X %08X %s" % (i, 4 * i, f, idc.get_func_name(f)))
