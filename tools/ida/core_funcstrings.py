# prints, per function in a range, the strings it references (client: edit RANGE before running)
import idautils, idc, ida_funcs, ida_bytes, ida_nalt
RANGE = (0xb3c30, 0xc10c0)
for f in idautils.Functions(*RANGE):
    fn = ida_funcs.get_func(f)
    ss = []
    for h in idautils.FuncItems(f):
        for x in idautils.DataRefsFrom(h):
            if ida_bytes.is_strlit(ida_bytes.get_flags(x)):
                s = idc.get_strlit_contents(x)
                if s and b'.cpp' not in s:
                    ss.append(s.decode('latin1')[:60])
    print('%x %d %s | %s' % (f, fn.end_ea - fn.start_ea, idc.get_func_name(f), ' ; '.join(dict.fromkeys(ss))[:400]))
