import idc
# print dwords at addresses listed in research/audio/rd_req.txt -> research/audio/rd_out.txt
B = r"D:\projects\project poland rewrite\research\audio"
with open(B + r"\rd_out.txt", "w") as fo:
    for line in open(B + r"\rd_req.txt"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        ea = int(line, 16)
        fo.write("%X %08X %s\n" % (ea, idc.get_wide_dword(ea), idc.get_name(ea)))
