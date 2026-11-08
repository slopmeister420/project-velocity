# physics agent stage 4b: AddCompartmentObject 0x1DFF60 is thiscall(manager, params)
import idc
print(idc.SetType(0x1DFF60, "__int64 __thiscall f(void *this, int a3);"), idc.get_type(0x1DFF60))
