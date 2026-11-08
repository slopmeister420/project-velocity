# physics agent stage 4b: E::PhysicsObjectParticleTracker 0x1E3CB0 is thiscall(obj, setup info)
import idc
print(idc.SetType(0x1E3CB0, "int __thiscall f(int this, int a3);"), idc.get_type(0x1E3CB0))
