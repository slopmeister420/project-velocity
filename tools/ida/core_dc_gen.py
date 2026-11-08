# usage (client side): python tools/ida/core_dc_gen.py [-o out.c] addr [addr...]  -> decompiles in IDA
# with -o the worker writes the text straight to the file (no response-size cap); assert noise is stripped.
import sys, os, subprocess
args = sys.argv[1:]
out = None
if args and args[0] == "-o":
    out = os.path.abspath(args[1])
    args = args[2:]
addrs = [int(a, 16) for a in args]
here = os.path.dirname(os.path.abspath(__file__))
body = "import ida_hexrays, ida_funcs, idc, re\n"
body += "ADDRS = %r\nOUT = %r\n" % (addrs, out)
body += r'''
NOISE = re.compile(r"__debugbreak|^\s*[{}]\s*$|off_6C18D8 != nullptr|InputDevice__3DDC30\(1\);|memset\(.*0xCC|= off_6C18D8\(|if \( v[0-9]+ == 3 \)|(HI|LO)WORD\(v.*= 257;|BYTE[0-9]?\(v.*= 1;|^\s*$")
parts = []
for a in ADDRS:
    f = ida_funcs.get_func(a)
    parts.append('=' * 20 + ' %x %s' % (a, idc.get_func_name(a)))
    try:
        txt = str(ida_hexrays.decompile(f.start_ea))
        parts.extend(l for l in txt.split('\n') if not NOISE.search(l))
    except Exception as e:
        parts.append('ERR %s' % e)
text = '\n'.join(parts)
if OUT:
    open(OUT, 'w', encoding='utf-8').write(text)
    print('wrote', OUT, len(text))
else:
    print(text)
'''
p = os.path.join(here, "core_tmp_dc.py")
open(p, "w").write(body)
root = os.path.dirname(os.path.dirname(here))
subprocess.run([os.path.join(root, ".venv-ida", "Scripts", "python"), os.path.join(root, "tools", "mcp.py"), "py", p])
