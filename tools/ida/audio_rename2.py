import idc, ida_name
N = {0x4C2A0: "VsArray::operator[]", 0x4C210: "VsBaseCamera::GetPositionInline", 0x54E30: "VsHashedString::VsHashedString",
     0x54E70: "VsHashedString::operator=", 0x54EB0: "VsVector4::Negated", 0x56940: "VsDestructorList::Add",
     0x56910: "VsDestructorList::RunAll", 0x569C0: "VsDestructorList::Count", 0x569E0: "VsDestructorList::RunTo",
     0x56A20: "VsDestructorList::RunOne", 0x568D0: "VsDestructorList::VsDestructorList", 0x568A0: "VsDestructorEntry::Clear"}
for ea, n in N.items():
    idc.set_name(ea, n, ida_name.SN_NOWARN | ida_name.SN_NOCHECK)
print("ok")
