"""rebuild / re-name research/default.xbe.i64 headless via idalib.

usage (below-normal priority, nothing else may hold the idb, port 8745 server stopped):
  py=".venv-ida/Scripts/python.exe"   (use the absolute path with lowprio)
  $py tools/lowprio.py $py tools/ida/rebuild_idb.py [--harvest-only] [--salvage <damaged.i64>]

steps
  1. (optional) --salvage: pull every user name out of a damaged idb's id0 b-tree into
     research/ida_salvaged_names.tsv (tools/ida/salvage_names.py).
  2. harvest names (pure python): funcmap stems `<FileStem>__<ADDR>`, research/movie_names.tsv,
     tools/ida/fe_rename*.tsv, research/ida_salvaged_names.tsv, and every `/// <File>.cpp 0x<ADDR>`
     port citation in crates/**.rs (`Type::PascalFn` inside an impl block, else `PascalFn`).
     precedence: citation > salvaged > tsv dumps > funcmap stem. a citation that disagrees with a
     salvaged real name keeps the salvaged one; two citations that disagree (and no salvaged name)
     keep the stem. every disagreement goes to research/ida_rebuild_conflicts.txt.
     the plan is written to research/ida_rebuild_names.tsv.
  3. if research/default.xbe.i64 is missing, create it from research/default.xbe with full
     auto-analysis; otherwise open it (re-run: only dummy `sub_`/stem names are overwritten).
  4. apply names (missing funcmap functions are created first), save, close cleanly.
"""
import glob, os, re, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
RES = os.path.join(ROOT, "research")
XBE = os.path.join(RES, "default.xbe")
IDB = XBE + ".i64"
GAME_LO, GAME_END = 0x11000, 0x3DDD20
AUTO = re.compile(r"^(sub|loc|locret|jpt|def|off|dword|byte|word|qword|unk|asc|stru|flt|dbl|xmmword|j|nullsub|algn|seg)_[0-9A-Fa-f]+$")
STEM = re.compile(r"^\w+__[0-9A-F]+$")


def norm(n):
    return re.sub(r"[^a-z0-9]", "", n.lower())


def pascal(n):
    if "_" not in n and n[:1].isupper():
        return n
    return "".join(p[:1].upper() + p[1:] for p in n.split("_") if p) or n


def stem_of(src):
    return re.sub(r"\W", "_", src.replace("\\", "/").split("/")[-1].rsplit(".", 1)[0])


def read_tsv(path, ai, ni):
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8", errors="replace"):
        c = line.rstrip("\r\n").split("\t")
        if len(c) > max(ai, ni) and re.fullmatch(r"(0x)?[0-9A-Fa-f]+", c[ai].strip()) and c[ni].strip():
            out[int(c[ai].strip(), 16)] = c[ni].strip()
    return out


CITE = re.compile(r"^\s*///\s*`?([\w\-./\\]+\.cpp)`?\s+0x([0-9A-Fa-f]{4,8})\b")
FN = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?((const|async|unsafe|default|extern\s+\"[^\"]*\")\s+)*fn\s+(\w+)")
IMPL = re.compile(r"^\s*(unsafe\s+)?impl\b\s*(<[^{]*?>)?\s*(.+?)\s*(where\b.*)?\{?\s*$")
TRAIT = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(unsafe\s+)?trait\s+\w+")
STRIP = re.compile(r"\"(\\.|[^\"\\])*\"|'(\\.|[^'\\])'|//.*$")


def impl_type(head):
    head = head.split(" for ")[-1].strip()
    head = re.sub(r"<.*", "", head).strip("&* ")
    head = head.replace("mut ", "").split("::")[-1]
    return head if re.fullmatch(r"[A-Za-z_]\w*", head) else None


def harvest_citations():
    cites = {}  # ea -> list of (name, file, where)
    for path in glob.glob(os.path.join(ROOT, "crates", "**", "*.rs"), recursive=True):
        rel = os.path.relpath(path, ROOT).replace("\\", "/")
        depth, stack, pending, pending_hdr = 0, [], [], None
        for ln, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
            m = CITE.match(line)
            s = line.strip()
            if m:
                pending.append((m.group(1), int(m.group(2), 16)))
            elif pending and (not s or s.startswith("///") or s.startswith("#[") or s.startswith("//")):
                pass
            elif pending:
                f = FN.match(line)
                if f:
                    t = stack[-1][0] if stack else None
                    nm = pascal(f.group(5))
                    nm = "%s::%s" % (t, nm) if t else nm
                    for src, ea in pending:
                        cites.setdefault(ea, []).append((nm, src, "%s:%d" % (rel, ln)))
                pending = []
            code = STRIP.sub("", line)
            if pending_hdr is not None and "{" in code:
                stack.append((pending_hdr, depth))
                pending_hdr = None
            im = IMPL.match(code)
            if im and not FN.match(code):
                t = impl_type(im.group(3))
                if "{" in code:
                    stack.append((t, depth))
                else:
                    pending_hdr = t
            elif TRAIT.match(code):
                if "{" in code:
                    stack.append((None, depth))
                else:
                    pending_hdr = None
            depth += code.count("{") - code.count("}")
            while stack and depth <= stack[-1][1]:
                stack.pop()
    return cites


def plan():
    conflicts = []
    funcmap, stems = {}, {}
    for line in open(os.path.join(RES, "funcmap.tsv")):
        c = line.rstrip("\r\n").split("\t")
        if len(c) >= 3 and c[0] and c[2] and "xbox-dec03" not in c[2]:
            ea = int(c[0], 16)
            funcmap[ea] = c[2]
            if GAME_LO <= ea < GAME_END:
                stems[ea] = "%s__%X" % (stem_of(c[2]), ea)
    dumps = {}
    for p, ai, ni in [(os.path.join(RES, "movie_names.tsv"), 0, 2)] + \
            [(p, 0, 1) for p in sorted(glob.glob(os.path.join(ROOT, "tools", "ida", "*rename*.tsv")))]:
        for ea, n in read_tsv(p, ai, ni).items():
            if ea in dumps and norm(dumps[ea]) != norm(n):
                conflicts.append("%X\tdump-vs-dump\t%s\t%s\t(%s)" % (ea, dumps[ea], n, os.path.basename(p)))
                continue
            dumps[ea] = n
    salv = {ea: n for ea, n in read_tsv(os.path.join(RES, "ida_salvaged_names.tsv"), 0, 1).items()
            if not AUTO.match(n)}
    cites = harvest_citations()
    final, origin = dict(stems), {ea: "stem" for ea in stems}
    for ea, n in dumps.items():
        s = salv.get(ea)
        if s and not STEM.match(s) and norm(s) != norm(n):
            conflicts.append("%X\tdump-vs-salvaged\tkeep %s\tdump %s" % (ea, s, n))
            continue
        final[ea], origin[ea] = n, "dump"
    for ea, n in salv.items():
        if origin.get(ea) == "dump" and STEM.match(n):
            continue
        final[ea], origin[ea] = n, "salvaged"
    for ea, lst in sorted(cites.items()):
        names = sorted({n for n, _, _ in lst}, key=lambda n: (-n.count("::"), n))
        where = "; ".join("%s@%s" % (n, w) for n, _, w in lst)
        for _, src, w in lst:
            fm = funcmap.get(ea)
            if fm and stem_of(fm).lower() != stem_of(src).lower():
                conflicts.append("%X\tcite-file-mismatch\tfuncmap %s\tcite %s @%s" % (ea, fm, src, w))
        # citations agree if one normalised name is a suffix of all others (class-less vs Class::)
        best = names[0]
        if not all(norm(best).endswith(norm(n)) for n in names):
            if origin.get(ea) != "salvaged" or STEM.match(final[ea]):
                conflicts.append("%X\tcite-vs-cite\tkeep %s\t%s" % (ea, final.get(ea, "-"), where))
                continue
            best = None
        cur = final.get(ea)
        if cur and origin.get(ea) in ("salvaged", "dump") and not STEM.match(cur):
            if best is None or not norm(cur).endswith(norm(best).split("::")[-1] if "::" in best else norm(best)) \
                    and not norm(cur).endswith(norm(best.split("::")[-1])):
                conflicts.append("%X\tcite-vs-%s\tkeep %s\t%s" % (ea, origin[ea], cur, where))
            continue  # salvaged/dump spelling is the original's; keep it
        if best:
            final[ea], origin[ea] = best, "cite"
    # duplicate names across addresses: suffix the later ones with the address
    seen = {}
    for ea in sorted(final):
        n = final[ea]
        if n in seen:
            conflicts.append("%X\tdup-name\t%s also at %X -> %s_%X" % (ea, n, seen[n], n, ea))
            final[ea] = "%s_%X" % (n, ea)
        else:
            seen[n] = ea
    with open(os.path.join(RES, "ida_rebuild_names.tsv"), "w") as o:
        for ea in sorted(final):
            o.write("%X\t%s\t%s\n" % (ea, final[ea], origin[ea]))
    with open(os.path.join(RES, "ida_rebuild_conflicts.txt"), "w") as o:
        o.write("# addr\tkind\tdetail  (salvaged = name in the pre-corruption idb; applied name is 'keep')\n")
        o.write("\n".join(conflicts) + "\n")
    counts = {}
    for v in origin.values():
        counts[v] = counts.get(v, 0) + 1
    print("plan", len(final), counts, "citations", len(cites), "conflicts", len(conflicts))
    return final, origin, funcmap


def apply(final, origin, funcmap):
    import idapro
    fresh = not os.path.exists(IDB)
    if idapro.open_database(XBE if fresh else IDB, True) != 0:
        sys.exit("open failed")
    import ida_auto, ida_funcs, ida_name, ida_bytes, idc, ida_loader
    ida_auto.auto_wait()
    made = 0
    for ea in sorted(funcmap):
        if GAME_LO <= ea < GAME_END and ida_funcs.get_func(ea) is None and ida_bytes.is_loaded(ea):
            if ida_funcs.add_func(ea):
                made += 1
    ida_auto.auto_wait()
    ok = fail = skip = 0
    flags = ida_name.SN_NOWARN | ida_name.SN_NOCHECK
    failed = []
    for ea, n in sorted(final.items()):
        if not ida_bytes.is_loaded(ea) and idc.get_segm_name(ea) == "":
            fail += 1
            failed.append("%X\tunmapped\t%s" % (ea, n))
            continue
        cur = ida_name.get_name(ea)
        if cur == n:
            skip += 1
            continue
        if not fresh and cur and not AUTO.match(cur) and not STEM.match(cur) and origin[ea] != "cite":
            skip += 1
            continue
        if ida_name.set_name(ea, n, flags):
            ok += 1
        else:
            fail += 1
            failed.append("%X\tset_name-failed\t%s" % (ea, n))
    if failed:
        with open(os.path.join(RES, "ida_rebuild_conflicts.txt"), "a") as o:
            o.write("\n".join(failed) + "\n")
    print("funcs created", made, "renamed", ok, "unchanged", skip, "failed", fail)
    idapro.close_database(True)


def main():
    a = sys.argv[1:]
    if "--salvage" in a:
        src = a[a.index("--salvage") + 1]
        subprocess.check_call([sys.executable, os.path.join(ROOT, "tools", "ida", "salvage_names.py"), src,
                               os.path.join(RES, "ida_salvaged_names.tsv")])
    final, origin, funcmap = plan()
    if "--harvest-only" not in a:
        apply(final, origin, funcmap)


if __name__ == "__main__":
    main()
