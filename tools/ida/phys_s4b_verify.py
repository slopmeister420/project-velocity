# physics agent stage 4b: are the stage-4b prototypes in the idb?
import idc
for a in [0x1E4C30, 0x1A89F0, 0x1ADF00, 0x378BD0, 0x37EF70, 0x1E3CB0]:
    print(hex(a), idc.get_func_name(a), idc.get_type(a))
