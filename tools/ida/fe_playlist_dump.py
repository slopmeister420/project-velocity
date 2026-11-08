# frontend agent: decompile the AudioPlaylistScreen range of FEQuickRaceTextScreen.cpp to research/fe_playlist_dc.c
import idautils, ida_hexrays, idc
out = open(r"D:\projects\project poland rewrite\research\fe_playlist_dc.c", "w")
for f in idautils.Functions(0x22B040, 0x22E850):
    out.write("//// %08X %s\n" % (f, idc.get_func_name(f)))
    try:
        out.write(str(ida_hexrays.decompile(f)) + "\n")
    except Exception as e:
        out.write("// fail %s\n" % e)
out.close()
print("ok")
