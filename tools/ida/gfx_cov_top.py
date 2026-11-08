import idautils, idc, ida_funcs, ida_nalt
tops=[0x1306E0,0x145A80,0x599E0,0x11CF70,0x2D3680,0x5A710,0x145620,0x1123E0]
out=open(r"D:\projects\project poland rewrite\research\gfx_cov_top.txt","w")
for a in tops:
    f=ida_funcs.get_func(a)
    strs=[];calls=[]
    for h in idautils.FuncItems(a):
        for x in [idc.get_operand_value(h,i) for i in (0,1)]:
            s=idc.get_strlit_contents(x) if x>0x10000 and idc.is_data(idc.get_full_flags(x)) else None
            if s and len(strs)<5: strs.append(s.decode('latin1')[:40])
        if idc.print_insn_mnem(h)=="call":
            n=idc.get_name(idc.get_operand_value(h,0))
            if n and n not in calls and len(calls)<8: calls.append(n)
    cs=[idc.get_func_name(r) for r in idautils.CodeRefsTo(a,0)][:3]
    out.write("%X\t%s\tstr=%s\tcalls=%s\tcallers=%s\n"%(a,idc.get_name(a),strs,calls,cs))
out.close()
