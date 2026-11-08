import idc
# static colour ctors -> research/ai_dump/name_colour4.txt
out = open(r"D:\projects\project poland rewrite\research\ai_dump\name_colour4.txt", "w")
for ea in [0x431AB7, 0x431AE1, 0x431B01, 0x431BE4]:
    x = ea
    for i in range(8):
        x = idc.prev_head(x)
    out.write("==== %X\n" % ea)
    while x <= ea + 8:
        out.write("%X  %s\n" % (x, idc.GetDisasm(x)))
        x = idc.next_head(x)
out.close()
