import idc
ea=0x95980
while ea<0x959c0:
    d=idc.GetDisasm(ea)
    print(hex(ea), d)
    ea=idc.next_head(ea)
