# physics agent stage 4b: compound-object mass/inertia helpers take the object pointer in ecx
import idc
print(idc.SetType(0x379660, "float *__thiscall f(_DWORD *this, float *a3, float a4);"), idc.get_type(0x379660))
print(idc.SetType(0x379050, "unsigned __int64 __thiscall f(_DWORD *this, _DWORD *a3, float *a4, float a5);"), idc.get_type(0x379050))
