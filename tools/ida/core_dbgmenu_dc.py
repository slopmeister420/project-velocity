import idc, ida_hexrays, idautils, re
# decompile the pause / tweaks menu regions + DebugMenu.cpp range into research/core_dbgmenu_dc.c
base = r"D:\projects\project poland rewrite\research"
NOISE = re.compile(r"__debugbreak|^\s*[{}]\s*$|off_6C18D8 != nullptr|InputDevice__3DDC30\(1\);|memset\(.*0xCC|= off_6C18D8\(|if \( v[0-9]+ == 3 \)|(HI|LO)WORD\(v.*= 257;|BYTE[0-9]?\(v.*= 1;|^\s*$")
RANGES = [(0x000AF740, 0x000B2BD0), (0x00082600, 0x00083000)]
out = open(base + r"\core_dbgmenu_dc.c", "w", encoding="utf-8")
for s, e in RANGES:
    for f in idautils.Functions(s, e):
        out.write("\n==== %x %s\n" % (f, idc.get_func_name(f)))
        try:
            txt = str(ida_hexrays.decompile(f))
            out.write("\n".join(l for l in txt.split("\n") if not NOISE.search(l)) + "\n")
        except Exception as ex:
            out.write("ERR %s\n" % ex)
out.close()
print("ok")
