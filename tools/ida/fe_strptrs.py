# frontend agent: print C strings pointed to by dwords in fe_peek_args.txt ranges (start count)
import idc
a = open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split()
for i in range(0, len(a), 2):
    s, n = int(a[i], 16), int(a[i+1])
    for k in range(n):
        p = idc.get_wide_dword(s + 4*k)
        print("%08X -> %08X %r" % (s + 4*k, p, idc.get_strlit_contents(p) if p else None))
