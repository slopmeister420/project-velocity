# physics agent: print the C strings at a few addresses
import idc
out = []
for a in (0x64243C, 0x631A98, 0x642430):
    s = idc.get_strlit_contents(a, -1, 0)
    out.append("%X %r" % (a, s))
print("\n".join(out))
