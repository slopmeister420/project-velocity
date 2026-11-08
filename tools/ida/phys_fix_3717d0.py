# physics agent: 0x3717D0 returns the steer angle in st0 (decompiler had it as dx:ax)
import idc
print(idc.SetType(0x3717D0, "float __thiscall f(float *this, float *target);"))
