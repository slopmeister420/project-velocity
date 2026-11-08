# usage (client side): python tools/ida/core_xs_gen.py addr [addr...]  -> funcs referencing each address
import sys, os, subprocess
addrs = [int(a, 16) for a in sys.argv[1:]]
here = os.path.dirname(os.path.abspath(__file__))
body = "import idautils, ida_funcs, idc\n"
body += "for a in %r:\n" % addrs
body += """    fs = []
    for x in idautils.XrefsTo(a):
        f = ida_funcs.get_func(x.frm)
        fs.append('%x %s @%x' % (f.start_ea, idc.get_func_name(f.start_ea), x.frm) if f else '?@%x' % x.frm)
    print(hex(a), repr(idc.get_strlit_contents(a) or b'')[:60], '|', '; '.join(fs))
"""
p = os.path.join(here, "core_tmp_xs.py")
open(p, "w").write(body)
root = os.path.dirname(os.path.dirname(here))
subprocess.run([os.path.join(root, ".venv-ida", "Scripts", "python"), os.path.join(root, "tools", "mcp.py"), "py", p])
