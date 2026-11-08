# physics agent stage 4b: give the two functions both named E::PhysicsObjectParticleTracker unique names; list callers
import idc, idautils, ida_funcs
out = set()
for a, n in [(0x1E3A60, "PhysicsObjectParticleTracker_1E3A60"), (0x1E3CB0, "PhysicsObjectParticleTracker_SetupMovable")]:
    print(hex(a), idc.set_name(a, n, idc.SN_NOWARN))
    out.add(a)
    for x in idautils.CodeRefsTo(a, 0):
        g = ida_funcs.get_func(x)
        if g and 0x1DE730 <= g.start_ea < 0x1E6130:
            out.add(g.start_ea)
open(r"D:\projects\project poland rewrite\research\phys_addrs.txt", "w").write("\n".join("%X" % a for a in sorted(out)) + "\n")
print(sorted(hex(a) for a in out))
