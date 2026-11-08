# physics agent stage 4b: 0x1ADF00 passes the frame dt (float) to every active object's update 0x1E4C30
import idc
print(idc.SetType(0x1ADF00, "int __thiscall f(_DWORD *this, float a2);"), idc.get_type(0x1ADF00))
