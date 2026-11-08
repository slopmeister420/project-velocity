# fix float-returning prototypes used by elParticle so the core decompiles cleanly
import idc, ida_typeinf
T = {
    0x320C60: "float __cdecl VsRandRangeF(float lo, float hi);",
    0x2D0520: "float __thiscall elParticleControlCurve_eval(float *this, float t);",
    0x57200: "float __thiscall VsPlane_distance(float *this, float *p);",
    0x2D04D0: "float __thiscall elVec3_length(float *this);",
}
for a, t in T.items():
    ok = idc.SetType(a, t)
    print(hex(a), ok)
