# signatures of every core-scope function (research/core_scope.txt, one hex addr per line):
# addr, size, mnemonic-sequence hash (operands masked to their types), net flag, callees, strings
# -> research/core_func_sig.tsv.  net = name/strings/callees name the network layer (net__*, *Net*, XOnline,
# Network), or every code caller is net (propagated to a fixed point).
import hashlib, re
import idautils, idc, ida_funcs
R = r"D:/projects/project poland rewrite/research/"
scope = [int(l, 16) for l in open(R + "core_scope.txt") if l.strip()]
NETRX = re.compile(r"NetData|ActorNet|NetPeer|XOnline|XNet|Network(Game|Manager|Message|Peer|MatchMaking)")
regs = []
for l in open(R + "objregions.tsv"):
    p = l.rstrip("\r\n").split("\t")
    if len(p) >= 3:
        regs.append((int(p[0], 16), int(p[1], 16), p[2]))


def region(ea):
    for s, e, f in regs:
        if s <= ea < e:
            return f
    return ""


def is_net_file(f):
    # the network layer proper; NetworkLogger is called by the local timeline code too
    return (f.startswith("src\\Network\\") and "NetworkLogger" not in f) or "Online" in f or "xbNetwork" in f


BORING = re.compile(r"^(InputDevice__3DDC[36]0|MEM_|operator_new|off_6C18D8|sub_3D|sub_3E|VsVector4|Vs[A-Z][a-z]+$)")
info = {}
for a in scope:
    f = ida_funcs.get_func(a)
    if not f:
        continue
    seq, callees, strs = [], set(), []
    ncall, nnet = 0, 0
    for ea in idautils.FuncItems(a):
        m = idc.print_insn_mnem(ea)
        ops = []
        for i in range(3):
            t = idc.get_operand_type(ea, i)
            if t == 0:
                break
            ops.append(str(t))
        seq.append(m + ":" + ",".join(ops))
        if m == "call":
            t = idc.get_operand_value(ea, 0)
            n = idc.get_name(t)
            if n:
                callees.add(n)
                if not BORING.match(n):
                    ncall += 1
                    nnet += is_net_file(region(t))
        for x in idautils.DataRefsFrom(ea):
            s = idc.get_strlit_contents(x, -1, 0)
            if s and len(s) > 2:
                strs.append(s.decode("latin-1")[:40])
    callers = set()
    for x in idautils.CodeRefsTo(a, 0):
        g = ida_funcs.get_func(x)
        if g:
            callers.add(g.start_ea)
    nm = idc.get_name(a)
    text = nm + " " + " ".join(strs)
    net = bool(NETRX.search(text)) or (ncall > 0 and 2 * nnet >= ncall)
    info[a] = dict(size=f.end_ea - f.start_ea, h=hashlib.md5("|".join(seq).encode()).hexdigest()[:12],
                   net=net, callers=callers, callees=callees, strs=strs)
# propagate: a function only called from net functions is net
changed = True
while changed:
    changed = False
    for a, d in info.items():
        if not d["net"] and d["callers"] and all(info.get(c, {}).get("net") or
                                                (c not in info and is_net_file(region(c)))
                                                for c in d["callers"]):
            d["net"] = True
            changed = True
out = []
for a in scope:
    d = info.get(a)
    if not d:
        continue
    out.append("%08X\t%d\t%s\t%d\t%s\t%s" % (a, d["size"], d["h"], d["net"], ",".join(sorted(d["callees"]))[:200],
                                          "|".join(d["strs"][:4]).replace("\t", " ").replace("\n", " ")))
open(R + "core_func_sig.tsv", "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(out), sum(1 for d in info.values() if d["net"]))
