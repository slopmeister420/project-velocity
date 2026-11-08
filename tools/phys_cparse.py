# physics agent: Hex-Rays C subset tokenizer/parser shared by the translators
import re
TYPES = ["unsigned __int8", "unsigned __int16", "unsigned int", "unsigned __int64", "__int64", "__int16",
         "__int8", "_DWORD", "_BYTE", "_WORD", "_QWORD", "float", "double", "int", "char", "bool", "void",
         "unsigned __int8 *", "_BYTE *", "_DWORD *", "float *", "int *", "char *", "void *", "double *",
         "__int16 *", "_WORD *", "unsigned int *", "const char *", "VehicleEntity", "_UNKNOWN", "signed int", "unsigned __int16", "__int32", "unsigned __int32", "__int32 *", "_QWORD", "signed __int64"]
TYPES.sort(key=len, reverse=True)
cast_re = "|".join(re.escape(t) for t in TYPES)
TOK = re.compile(r"\s*(?:(?P<cast>\((?:%s)\s*\**\s*\)|\((?:const\s+)?(?:unsigned\s+)?[A-Za-z_]\w*\s*\*+\s*\))|(?P<num>\d+\.\d*(?:e[+-]?\d+)?f?|\d+e[+-]?\d+f?|0x[0-9A-Fa-f]+u?|\d+u?)|(?P<str>\"(?:[^\"\\]|\\.)*\")|(?P<id>[A-Za-z_]\w*)|(?P<op>\+\+|--|->|<<=|>>=|<<|>>|<=|>=|==|!=|&&|\|\||\+=|-=|\*=|/=|\|=|&=|\^=|[-+*/%%(),\[\];=<>!&|^~?:{}.]))" % cast_re)

def tokenize(s):
    # drop // comments but not "//" inside string literals
    s = re.sub(r"(\"(?:[^\"\\\n]|\\.)*\")|//[^\n]*", lambda m: m.group(1) or "", s)
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
        while p.peek()[0] == "str":
            v = v[:-1] + p.take()[1][1:]
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
    if v == "switch":
        p.take()
        p.expect("(")
        c = p_expr(p)
        p.expect(")")
        p.expect("{")
        cases = []
        while p.peek()[1] != "}":
            if p.peek()[1] == "case":
                p.take()
                val = p_cond(p)
                p.expect(":")
                cases.append(([val], []))
            elif p.peek()[1] == "default":
                p.take()
                p.expect(":")
                cases.append((["default"], []))
            else:
                cases[-1][1].append(p_stmt(p))
        p.take()
        return ("switch", c, cases)
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

