import idc, idautils
# instructions in VsAudio.cpp (0x47A00..0x4FD50) touching [reg+1E4h] (VsAudio +484 "Debug Txt on")
o = open(r"D:\projects\project poland rewrite\research\audio\tweak_debugtxt.txt", "w")
for h in idautils.Heads(0x47A00, 0x4FD50):
    d = idc.GetDisasm(h)
    if "+1E4h]" in d:
        o.write("%x %s : %s\n" % (h, idc.get_func_name(h), d))
o.close()
print("ok")
