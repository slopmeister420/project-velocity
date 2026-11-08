import idautils, idc, ida_funcs, ida_hexrays
out = open(r"D:\projects\project poland rewrite\research\core_camlerp3.txt", "w")
def name(ea):
    return idc.get_func_name(ea) or hex(ea)
def dec(f):
    try:
        out.write("\n==== %x %s\n%s\n" % (f, name(f), str(ida_hexrays.decompile(f))))
    except Exception as e:
        out.write("\n==== %x fail %s\n" % (f, e))
f = idc.get_func_attr(0x16e301, idc.FUNCATTR_START)
dec(f)
# functions in follow camera / vehicle entity regions that read the frame timer lerp (+0x20 of the timer global)
out.write("\n== xrefs to frame timer users\n")
for x in idautils.CodeRefsTo(0x57580, 0):
    pass
out.close()
