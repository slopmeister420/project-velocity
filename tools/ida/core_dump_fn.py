import ida_hexrays, ida_funcs, idc
# dump full decompilations of the listed functions to research/core_<addr>.c
ADDRS = [0x5C9C0]
for a in ADDRS:
    f = ida_funcs.get_func(a)
    try:
        c = str(ida_hexrays.decompile(f))
    except Exception as e:
        c = "error %s" % e
    open(r"D:\projects\project poland rewrite\research\core_%X.c" % a, "w").write(c)
print("ok")
