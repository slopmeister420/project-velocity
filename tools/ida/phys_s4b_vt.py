# physics agent stage 4b: entries of the physics-object rigid body vtable 0x64B774 (and CRigidBody 0x675894)
import idc, ida_bytes
out = []
for vt in [0x64B774, 0x675894]:
    out.append("== %X" % vt)
    for i in range(12):
        f = ida_bytes.get_dword(vt + 4 * i)
        if not (0x11000 <= f < 0x3E0000):
            break
        out.append("  +%d %X %s" % (4 * i, f, idc.get_func_name(f)))
open(r"D:\projects\project poland rewrite\research\phys_s4b_vt.txt", "w").write("\n".join(out))
