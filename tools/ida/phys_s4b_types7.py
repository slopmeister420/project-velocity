# physics agent stage 4b: compound object vs triangle soup 0x378BD0 takes (contacts, count, verts, nverts, faces, normals, nfaces, float, float)
import idc
print(idc.SetType(0x378BD0, "int __thiscall f(int this, int a2, float *a3, float *a4, int a5, int *a6, float *a7, int a8, float a9, float a10);"), idc.get_type(0x378BD0))
