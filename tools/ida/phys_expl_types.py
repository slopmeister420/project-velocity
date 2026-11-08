# physics agent: explosion port - thiscall prototypes (edx was never an argument: retn 14h / 10h)
import idc
T = {
    0x1E1A20: "__int64 __thiscall PhysicsObjectManager__1E1A20(void *this, _DWORD *pos, int a4, int a5, float a6, int a7);",
    0x1B3500: "__int64 __thiscall PhysicsManager__1B3500(void *this, _DWORD *pos, int a4, int a5, int a6);",
}
for a, t in T.items():
    print(hex(a), idc.SetType(a, t))
