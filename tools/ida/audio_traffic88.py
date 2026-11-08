import idc, ida_funcs, ida_hexrays, idautils, re
# decompile VsAudio*.cpp (0x49280..0x56D90) and list lines touching the traffic array (VsAudio +40, 172-B entries)
OUT = r"D:\projects\project poland rewrite\research\audio\traffic88.txt"
ida_hexrays.init_hexrays_plugin()
with open(OUT, "w", encoding="utf-8") as fo:
    for ea in idautils.Functions(0x49280, 0x56D90):
        try:
            c = ida_hexrays.decompile(ea)
        except Exception:
            c = None
        if c is None:
            continue
        for ln in str(c).splitlines():
            if "172 *" in ln and ("+ 88" in ln or "+ 40)" in ln):
                fo.write("%x %s: %s\n" % (ea, idc.get_func_name(ea), ln.strip()))
