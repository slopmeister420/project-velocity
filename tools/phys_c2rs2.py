# physics agent: semi-automatic Hex-Rays (debug build) -> Rust translator for snippets.
# usage: phys_c2rs2.py <snippet.c> [--this EXPR=NAME ...] [--arr NAME] > out.rs
# Translates statements (assign, if/else, for/while/do, return, break/continue, goto as TODO) and
# expressions keeping the float model: `(float)(a op b)` -> f32 op; uncast op on floats -> f64 (x87);
# Vs* math returns f64; stores to float lvalues round. Memory: `*(T *)(BASE + OFF)` where BASE matches
# a --this mapping becomes NAME.f(OFF) / NAME.i(OFF) / NAME.b(OFF) (writes: NAME.set_f(OFF, v)).
# Output is meant for review, not blind use.
import re, sys

args = sys.argv[1:]
src = open(args[0], encoding="utf-8").read()
BASES = {}
INTS = set()
for a in args[1:]:
    if a.startswith("INT:"):
        INTS.update(a[4:].split(","))
        continue
    if a.startswith("--this"):
        continue
    if "=" in a:
        k, v = a.split("=", 1)
        BASES[k.replace(" ", "")] = v

TYPES = ["unsigned __int8", "unsigned __int16", "unsigned int", "unsigned __int64", "__int64", "__int16",
         "__int8", "_DWORD", "_BYTE", "_WORD", "_QWORD", "float", "double", "int", "char", "bool", "void",
         "unsigned __int8 *", "_BYTE *", "_DWORD *", "float *", "int *", "char *", "void *", "double *",
         "__int16 *", "_WORD *", "unsigned int *", "const char *"]
TYPES.sort(key=len, reverse=True)
cast_re = "|".join(re.escape(t) for t in TYPES)
TOK = re.compile(r"\s*(?:(?P<cast>\((?:%s)\s*\**\s*\))|(?P<num>\d+\.\d*(?:e[+-]?\d+)?f?|\d+e[+-]?\d+f?|0x[0-9A-Fa-f]+u?|\d+u?)|(?P<str>\"(?:[^\"\\]|\\.)*\")|(?P<id>[A-Za-z_]\w*)|(?P<op>\+\+|--|->|<<=|>>=|<<|>>|<=|>=|==|!=|&&|\|\||\+=|-=|\*=|/=|\|=|&=|\^=|[-+*/%%(),\[\];=<>!&|^~?:{}.]))" % cast_re)

def tokenize(s):
    s = re.sub(r"//[^\n]*", "", s)
    out = []
    pos = 0
    while True:
        m = TOK.match(s, pos)
        if not m or m.end() == pos:
            if s[pos:].strip():
                raise Exception("tok at: %r" % s[pos:pos + 50])
            break
        pos = m.end()
        k = m.lastgroup
        out.append((k, m.group(k)))
    return out

class P:
    def __init__(self, t):
        self.t = t
        self.i = 0
    def peek(self, k=0):
        j = self.i + k
        return self.t[j] if j < len(self.t) else ("eof", None)
    def take(self):
        x = self.t[self.i]
        self.i += 1
        return x
    def accept(self, v):
        if self.peek()[1] == v:
            self.i += 1
            return True
        return False
    def expect(self, v):
        x = self.take()
        if x[1] != v:
            raise Exception("expected %r got %r near %r" % (v, x, self.t[self.i:self.i + 8]))

PREC = [["||"], ["&&"], ["|"], ["^"], ["&"], ["==", "!="], ["<", ">", "<=", ">="], ["<<", ">>"], ["+", "-"], ["*", "/", "%"]]

def p_expr(p):
    e = p_assign(p)
    while p.peek()[1] == ",":
        p.take()
        e = ("comma", e, p_assign(p))
    return e

def p_assign(p):
    l = p_cond(p)
    if p.peek()[1] in ("=", "+=", "-=", "*=", "/=", "|=", "&=", "^=", "<<=", ">>="):
        op = p.take()[1]
        r = p_assign(p)
        return ("assign", op, l, r)
    return l

def p_cond(p):
    c = p_bin(p, 0)
    if p.accept("?"):
        a = p_expr(p)
        p.expect(":")
        b = p_cond(p)
        return ("tern", c, a, b)
    return c

def p_bin(p, lvl):
    if lvl == len(PREC):
        return p_unary(p)
    l = p_bin(p, lvl + 1)
    while p.peek()[1] in PREC[lvl] and p.peek()[0] == "op":
        op = p.take()[1]
        r = p_bin(p, lvl + 1)
        l = ("bin", op, l, r)
    return l

def p_unary(p):
    k, v = p.peek()
    if k == "cast":
        p.take()
        ty = re.sub(r"\s+", " ", v[1:-1].strip())
        return ("cast", ty, p_unary(p))
    if v in ("-", "!", "~", "*", "&", "++", "--") and k == "op":
        p.take()
        return ("un", v, p_unary(p))
    return p_postfix(p)

def p_postfix(p):
    e = p_primary(p)
    while True:
        v = p.peek()[1]
        if v == "[":
            p.take()
            i = p_expr(p)
            p.expect("]")
            e = ("idx", e, i)
        elif v == "(":
            p.take()
            a = []
            if p.peek()[1] != ")":
                a.append(p_assign(p))
                while p.accept(","):
                    a.append(p_assign(p))
            p.expect(")")
            e = ("call", e, a)
        elif v in ("++", "--"):
            p.take()
            e = ("post", v, e)
        elif v in (".", "->"):
            p.take()
            e = ("field", e, p.take()[1])
        else:
            return e

def p_primary(p):
    k, v = p.take()
    if k == "num":
        return ("num", v)
    if k == "str":
        return ("str", v)
    if k == "id":
        return ("id", v)
    if v == "(":
        e = p_expr(p)
        p.expect(")")
        return e
    raise Exception("bad primary %r %r" % (k, v))

def p_stmt(p):
    k, v = p.peek()
    if v == "{":
        p.take()
        b = []
        while p.peek()[1] != "}":
            b.append(p_stmt(p))
        p.take()
        return ("block", b)
    if v == "if":
        p.take()
        p.expect("(")
        c = p_expr(p)
        p.expect(")")
        a = p_stmt(p)
        b = None
        if p.peek()[1] == "else":
            p.take()
            b = p_stmt(p)
        return ("if", c, a, b)
    if v == "for":
        p.take()
        p.expect("(")
        i = None if p.peek()[1] == ";" else p_expr(p)
        p.expect(";")
        c = None if p.peek()[1] == ";" else p_expr(p)
        p.expect(";")
        s = None if p.peek()[1] == ")" else p_expr(p)
        p.expect(")")
        return ("for", i, c, s, p_stmt(p))
    if v == "while":
        p.take()
        p.expect("(")
        c = p_expr(p)
        p.expect(")")
        return ("while", c, p_stmt(p))
    if v == "do":
        p.take()
        b = p_stmt(p)
        p.expect("while")
        p.expect("(")
        c = p_expr(p)
        p.expect(")")
        p.expect(";")
        return ("do", b, c)
    if v == "return":
        p.take()
        e = None if p.peek()[1] == ";" else p_expr(p)
        p.expect(";")
        return ("return", e)
    if v in ("break", "continue"):
        p.take()
        p.expect(";")
        return (v,)
    if v == "goto":
        p.take()
        l = p.take()[1]
        p.expect(";")
        return ("goto", l)
    if k == "id" and p.peek(1)[1] == ":" :
        p.take()
        p.take()
        return ("label", v)
    if v == ";":
        p.take()
        return ("empty",)
    e = p_expr(p)
    p.expect(";")
    return ("expr", e)

# ---------------- emission
FN = {"VsSin": ("vsmath::sin", "f64"), "VsCos": ("vsmath::cos", "f64"), "VsSqrt": ("vsmath::sqrt", "f64"),
      "VsFabs": ("vsmath::fabs", "f64"), "VsTan": ("vsmath::tan", "f64"), "VsAtan": ("vsmath::atan", "f64"),
      "VsAtan2": ("vsmath::atan2", "f64"), "VsACos": ("vsmath::acos", "f64"), "VsAsin": ("vsmath::asin", "f64"),
      "VsExp": ("vsmath::exp", "f64"), "VsPow": ("vsmath::pow", "f64"), "VsClamp": ("vsmath::clamp", "f32")}
DOUBLES = set()  # names of double locals

def key(e):
    return re.sub(r"\s+", "", show(e))

def show(e):
    k = e[0]
    if k == "num" or k == "id" or k == "str":
        return e[1]
    if k == "idx":
        return "%s[%s]" % (show(e[1]), show(e[2]))
    if k == "call":
        return "%s(%s)" % (show(e[1]), ",".join(show(a) for a in e[2]))
    if k == "bin":
        return "(%s%s%s)" % (show(e[2]), e[1], show(e[3]))
    if k == "un":
        return "%s%s" % (e[1], show(e[2]))
    if k == "cast":
        return "(%s)%s" % (e[1], show(e[2]))
    return str(e)

def split_base(e):
    """`base + off` -> (basekey, offexpr) when base is a known object."""
    if e[0] == "bin" and e[1] == "+":
        l, r = e[2], e[3]
        b, o = split_base(l)
        if b is not None:
            return b, (o + [r])
        b, o = split_base(r)
        if b is not None:
            return b, (o + [l])
    kk = key(e)
    if kk in BASES:
        return BASES[kk], []
    if e[0] == "cast":
        return split_base(e[2])
    return None, None

def emit_int(e):
    k = e[0]
    if k == "num":
        return e[1].rstrip("u")
    if k == "id":
        return e[1]
    if k == "bin":
        return "(%s %s %s)" % (emit_int(e[2]), e[1], emit_int(e[3]))
    if k == "cast":
        return emit_int(e[2])
    if k == "call" and e[1][0] == "id" and e[1][1] in ("LODWORD", "SLODWORD"):
        return "%s.to_bits() as i32" % emit_val(e[2][0], "f32")
    return "/*?*/" + show(e)

def deref(e):
    """*(T *)(...) -> (accessor, base, offset, type)"""
    if e[0] == "un" and e[1] == "*" and e[2][0] == "cast":
        ty = e[2][1].replace(" ", "")
        b, offs = split_base(e[2][2])
        if b is not None:
            off = " + ".join(emit_int(o) for o in offs) if offs else "0"
            if not re.match(r"^\d+$", off):
                off = "(%s) as usize" % off
            t = {"float*": "f", "_DWORD*": "i", "int*": "i", "_BYTE*": "b", "char*": "b", "unsignedint*": "i",
                 "unsigned__int8*": "b", "__int16*": "i16", "_WORD*": "i16"}.get(ty, "i")
            return b, off, t
    return None

def emit_val(e, want):
    s, t = emit(e)
    return conv(s, t, want)

def conv(s, t, want):
    if t == want or want is None:
        return s
    if want == "f64":
        return "(%s as f64)" % s
    if want == "f32":
        return "(%s as f32)" % s
    if want == "i32":
        if t in ("f32", "f64"):
            return "(%s as i32)" % s
        if t == "bool":
            return "(%s as i32)" % s
        return s
    if want == "bool":
        if t == "bool":
            return s
        if t in ("f32", "f64"):
            return "(%s != 0.0)" % s
        return "(%s != 0)" % s
    return s

def emit(e):
    """returns (rust, type) type in f32 f64 i32 bool ?"""
    k = e[0]
    d = deref(e)
    if d:
        b, off, t = d
        return ("%s.%s(%s)" % (b, t, off), {"f": "f32", "i": "i32", "b": "i32", "i16": "i32"}[t])
    if k == "num":
        v = e[1]
        if "." in v or "e" in v.lower() and not v.startswith("0x"):
            return (v.rstrip("f") if "." in v else v.rstrip("f") + ".0", "lit")
        return (v.rstrip("u"), "i32")
    if k == "id":
        n = e[1]
        if n in DOUBLES:
            return (n, "f64")
        if n in BASES:
            return (BASES[n], "?")
        if n in INTS:
            return (n, "i32")
        if re.match(r"^a\d+$", n) or re.match(r"^v\d+$", n):
            return (n, "f32")
        return (n, "?")
    if k == "idx":
        return ("%s[%s]" % (emit(e[1])[0], emit_int(e[2])), "f32")
    if k == "cast":
        ty = e[1]
        if ty == "float":
            inner = e[2]
            if inner[0] == "bin" and inner[1] in "+-*/":
                a = emit_val_lit(inner[2], "f32")
                b = emit_val_lit(inner[3], "f32")
                return ("(%s %s %s)" % (a, inner[1], b), "f32")
            s, t = emit(inner)
            if t == "lit":
                return (s + "f32", "f32")
            return (conv(s, t, "f32"), "f32")
        if ty == "double":
            return (emit_val_lit(e[2], "f64"), "f64")
        if ty in ("int", "__int64", "unsigned int", "__int16", "_DWORD"):
            s, t = emit(e[2])
            return (conv(s, t, "i32") if t != "lit" else s, "i32")
        return emit(e[2])
    if k == "un":
        op = e[1]
        if op == "-":
            s, t = emit(e[2])
            if t == "lit":
                return ("-%s" % s, t)
            return ("(-%s)" % s, t)
        if op == "!":
            return ("!%s" % emit_val(e[2], "bool"), "bool")
        return ("/*TODO %s*/" % show(e), "?")
    if k == "bin":
        op = e[1]
        if op in ("&&", "||"):
            return ("(%s %s %s)" % (emit_val(e[2], "bool"), op, emit_val(e[3], "bool")), "bool")
        if op in ("<", ">", "<=", ">=", "==", "!="):
            a, ta = emit(e[2])
            b, tb = emit(e[3])
            if "f64" in (ta, tb):
                return ("(%s %s %s)" % (lit(a, ta, "f64"), op, lit(b, tb, "f64")), "bool")
            if "f32" in (ta, tb):
                return ("(%s %s %s)" % (lit(a, ta, "f32"), op, lit(b, tb, "f32")), "bool")
            return ("(%s %s %s)" % (a, op, b), "bool")
        a, ta = emit(e[2])
        b, tb = emit(e[3])
        if ta in ("i32", "?") and tb in ("i32", "?", "lit") and not (tb == "lit" and "." in b):
            return ("(%s %s %s)" % (a, op, b), "i32")
        # uncast float op -> x87
        return ("(%s %s %s)" % (lit(a, ta, "f64"), op, lit(b, tb, "f64")), "f64")
    if k == "call":
        f = e[1]
        if f[0] == "id" and f[1] in FN:
            name, rt = FN[f[1]]
            return ("%s(%s)" % (name, ", ".join(emit_val_lit(a, "f32") for a in e[2])), rt)
        if f[0] == "id" and f[1] in ("LODWORD", "SLODWORD"):
            return ("%s.to_bits() as i32" % emit_val(e[2][0], "f32"), "i32")
        return ("%s(%s)" % (show(f), ", ".join(emit(a)[0] for a in e[2])), "?")
    if k == "tern":
        return ("(if %s { %s } else { %s })" % (emit_val(e[1], "bool"), emit(e[2])[0], emit(e[3])[0]), emit(e[2])[1])
    return ("/*TODO %s*/" % show(e), "?")

def lit(s, t, want):
    if t == "lit":
        return s + ("f32" if want == "f32" else "f64")
    if t == "i32" and want in ("f32", "f64"):
        return "(%s as %s)" % (s, want)
    return conv(s, t, want)

def emit_val_lit(e, want):
    s, t = emit(e)
    return lit(s, t, want)

def emit_assign(op, l, r, ind):
    d = deref(l)
    if d:
        b, off, t = d
        if op == "=":
            want = {"f": "f32", "i": "i32", "b": "i32", "i16": "i32"}[t]
            s, tt = emit(r)
            if t == "i" and tt in ("f32",):
                return "%s%s.set_u(%s, %s.to_bits());" % (ind, b, off, s)
            return "%s%s.set_%s(%s, %s);" % (ind, b, t, off, lit(s, tt, want))
        return "%s// TODO compound %s on %s" % (ind, op, show(l))
    ls, lt = emit(l)
    if l[0] == "call" and l[1][1] in ("LODWORD", "SLODWORD"):
        tgt = emit(l[2][0])[0]
        return "%s%s = f32::from_bits((%s) as u32);" % (ind, tgt, emit_val(r, "i32"))
    want = "f64" if lt == "f64" else ("f32" if lt == "f32" else None)
    s, tt = emit(r)
    if op == "=":
        return "%s%s = %s;" % (ind, ls, lit(s, tt, want) if want else s)
    return "%s%s %s %s;" % (ind, ls, op, lit(s, tt, want) if want else s)

def emit_stmt(s, ind):
    k = s[0]
    o = []
    if k == "block":
        for x in s[1]:
            o += emit_stmt(x, ind)
    elif k == "expr":
        e = s[1]
        if e[0] == "assign":
            o.append(emit_assign(e[1], e[2], e[3], ind))
        elif e[0] == "comma":
            o += emit_stmt(("expr", e[1]), ind)
            o += emit_stmt(("expr", e[2]), ind)
        else:
            o.append(ind + emit(e)[0] + ";")
    elif k == "if":
        o.append(ind + "if %s {" % emit_val(s[1], "bool"))
        o += emit_stmt(s[2], ind + "    ")
        if s[3] is not None:
            if s[3][0] == "if":
                sub = emit_stmt(s[3], ind)
                sub[0] = ind + "} else " + sub[0].lstrip()
                o += sub
                return o
            o.append(ind + "} else {")
            o += emit_stmt(s[3], ind + "    ")
        o.append(ind + "}")
    elif k == "for":
        o.append(ind + "// for (%s; %s; %s)" % (show(s[1]) if s[1] else "", show(s[2]) if s[2] else "", show(s[3]) if s[3] else ""))
        if s[1]:
            o += emit_stmt(("expr", s[1]), ind)
        o.append(ind + "while %s {" % (emit_val(s[2], "bool") if s[2] else "true"))
        o += emit_stmt(s[4], ind + "    ")
        if s[3]:
            o.append(ind + "    // step: " + show(s[3]))
        o.append(ind + "}")
    elif k == "while":
        o.append(ind + "while %s {" % emit_val(s[1], "bool"))
        o += emit_stmt(s[2], ind + "    ")
        o.append(ind + "}")
    elif k == "do":
        o.append(ind + "loop {")
        o += emit_stmt(s[1], ind + "    ")
        o.append(ind + "    if !%s { break; }" % emit_val(s[2], "bool"))
        o.append(ind + "}")
    elif k == "return":
        o.append(ind + "return%s;" % ("" if s[1] is None else " " + emit(s[1])[0]))
    elif k in ("break", "continue"):
        o.append(ind + k + ";")
    elif k == "goto":
        o.append(ind + "// TODO goto " + s[1])
    elif k == "label":
        o.append(ind + "// LABEL " + s[1] + ":")
    return o

SCAL = {}
for m in re.finditer(r"^\s*(double|float) (v\d+);", src, re.M):
    SCAL[m.group(2)] = "f64" if m.group(1) == "double" else "f32"
    if m.group(1) == "double":
        DOUBLES.add(m.group(2))
body = src
if "{" in src and re.search(r"^\w.*\(.*\)\s*$", src.split("\n")[0]):
    body = src[src.index("{"):]
body = "\n".join(l for l in body.split("\n") if not re.match(r"^\s*(float|double|int|_DWORD|__int64|char|unsigned|_BYTE|bool|void)\b[^=;(]*;", l) and "memset(" not in l)
p = P(tokenize(body))
stmts = []
while p.peek()[0] != "eof":
    stmts.append(p_stmt(p))
out = []
for s in stmts:
    out += emit_stmt(s, "    ")
txt = "\n".join(out)
head = ["    let mut %s: %s = 0.0;" % (n, t) for n, t in sorted(SCAL.items()) if re.search(r"\b%s\b" % n, txt)]
print("\n".join(head + out))
