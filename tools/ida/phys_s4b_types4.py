# physics agent stage 4b: 0x37EF70 takes two pointers (box test), not a by-value __int64
import idc
print(idc.SetType(0x37EF70, "unsigned __int64 __cdecl f(float *a1, float *a2);"), idc.get_type(0x37EF70))
