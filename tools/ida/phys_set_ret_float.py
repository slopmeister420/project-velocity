# physics agent: retype the return of the functions in research/_ret_float.txt (hex, one per line) to float
import ida_typeinf, ida_nalt, ida_hexrays, idc
R = r"D:\projects\project poland rewrite\research"
for line in open(R + r"\_ret_float.txt"):
    line = line.split("#")[0].strip()
    if not line:
        continue
    f = int(line, 16)
    tif = ida_typeinf.tinfo_t()
    if not ida_nalt.get_tinfo(tif, f):
        cf = ida_hexrays.decompile(f)
        tif = cf.type
    fi = ida_typeinf.func_type_data_t()
    tif.get_func_details(fi)
    fl = ida_typeinf.tinfo_t()
    fl.create_simple_type(ida_typeinf.BTF_FLOAT)
    fi.rettype = fl
    nt = ida_typeinf.tinfo_t()
    nt.create_func(fi)
    print(hex(f), ida_typeinf.apply_tinfo(f, nt, ida_typeinf.TINFO_DEFINITE), idc.get_type(f))
