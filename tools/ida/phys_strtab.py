# physics agent: dump a table of string pointers (args in research/phys_strtab_args.txt: addr count)
import idc, ida_bytes
a, n = open(r"D:\projects\project poland rewrite\research\phys_strtab_args.txt").read().split()
a, n = int(a, 16), int(n)
out = []
for i in range(n):
    p = ida_bytes.get_dword(a + 4 * i)
    s = idc.get_strlit_contents(p)
    out.append(s.decode("latin1") if s else "<%08X>" % p)
print(out)
