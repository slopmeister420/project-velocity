# physics agent stage 4b: thiscall prototypes of the physics-object functions (from retn sizes, edx unused)
import idc
T = {
    0x1E4C30: "_DWORD *__thiscall f(_DWORD *this, float dt);",
    0x1DEBB0: "char __thiscall f(_DWORD *this, _DWORD *a3, float *a4);",
    0x1E4710: "int __thiscall f(_DWORD *this);",
    0x1E3A60: "__int64 __thiscall f(_DWORD *this);",
    0x1E1F50: "void __cdecl f(float a2, float a3, float a4);",
    0x384C70: "__int64 __thiscall f(unsigned __int16 *this, int a3, float a4, float a5);",
    0x1E5270: "__int64 __thiscall f(_DWORD *this, _DWORD *a3, float *a4);",
    0x379660: "__int64 __thiscall f(int this, float *a3, float a4);",
    0x3847C0: "char __thiscall f(void *this, int a3, float a4, float a5, float a6, char a7);",
    0x379050: "unsigned __int64 __thiscall f(int this, _DWORD *a3, float *a4, float a5);",
}
for a, t in T.items():
    print(hex(a), idc.SetType(a, t), idc.get_type(a))
