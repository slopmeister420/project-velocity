import idc, idautils, ida_bytes, ida_hexrays
# core tweak details -> research/core_tweak_info.txt: vibrate effect table, contrast object, registration contexts
out = open(r"D:\projects\project poland rewrite\research\core_tweak_info.txt", "w", encoding="utf-8")
def s(ea):
    v = idc.get_strlit_contents(ea, -1, 0)
    return v.decode("latin-1") if v else "?"
out.write("== vibrate table 0x688C10\n")
for i in range(8):
    b = 0x688C10 + 16 * i
    out.write("%d name=%s motor=%08X int=%08X decay=%08X\n" % (i, s(ida_bytes.get_dword(b)), ida_bytes.get_dword(b + 4),
              ida_bytes.get_dword(b + 8), ida_bytes.get_dword(b + 12)))
out.write("== contrast 0x6F5C78 loaded=%d\n" % ida_bytes.is_loaded(0x6F5C78))
for a in range(0x6F5C78 + 96, 0x6F5C78 + 140, 4):
    for x in idautils.XrefsTo(a):
        out.write("  xr %X -> %X %s t%d\n" % (a, x.frm, idc.get_func_name(x.frm), x.type))
for x in idautils.XrefsTo(0x6F5C78):
    out.write("  xr base -> %X %s t%d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
def ctx(f, lo, hi, tag):
    try:
        t = str(ida_hexrays.decompile(f)).split("\n")
    except Exception as e:
        out.write("ERR %s\n" % e); return
    out.write("== %s %X %s\n" % (tag, f, idc.get_func_name(f)))
    for l in t:
        if any(k in l for k in lo):
            out.write(l + "\n")
ctx(0x17F60, ["167F30", "("], 0, "score caller")
ctx(0xC90A0, ["C9450", "Load", "("], 0, "heathaze caller")
ctx(0x165400, ["165640", "("], 0, "replay caller")
out.write("== vtable 62BB7C users\n")
vt = 0x62BB7C
for x in idautils.XrefsTo(vt - 0x0):
    out.write("  %X %s\n" % (x.frm, idc.get_func_name(x.frm)))
for k in range(-8, 12):
    out.write("  vt[%d] %X %s\n" % (k, ida_bytes.get_dword(vt + 4 * k), idc.get_func_name(ida_bytes.get_dword(vt + 4 * k))))
for a in [0x6D60B8, 0x6851F0, 0x685EF0, 0x685EF4, 0x6F5B7D, 0x749120, 0x6853F0]:
    out.write("== xr %X\n" % a)
    for x in idautils.XrefsTo(a):
        out.write("  %X %s t%d\n" % (x.frm, idc.get_func_name(x.frm), x.type))
out.close()
print("ok")
