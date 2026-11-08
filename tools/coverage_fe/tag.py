import os, re
from collections import Counter
root = r"D:\projects\project poland rewrite\research"
raw = {}
for l in open(os.path.join(root, "fe_cov_raw.tsv")):
    p = l.rstrip("\n").split("\t"); raw[p[0]] = (int(p[1]), p[2], p[3] == "1")
nm = {}
for l in open(os.path.join(root, "fe_cov_names.tsv")):
    p = l.rstrip("\n").split("\t")
    if len(p) >= 6 and p[0] in raw: nm[p[0]] = p[1:]
rows = []
for a, (sz, f, men) in raw.items():
    n, nc, ncall, src, strs = (nm.get(a) or ["", "0", "0", "", ""])
    stem = f.replace(".cpp", "")
    named = "__" not in n and n != ""
    if src and os.path.basename(src.replace("\\", "/")).replace(".cpp", "") != stem and src.endswith(".cpp"):
        tag = "mis-attributed(%s)" % os.path.basename(src.replace("\\", "/"))
    elif re.search(r"Online|Multiplayer|NetworkServer|Optimatch", f): tag = "online(XBL, cut)"
    elif "VsMovieStream" in f or f == "FontManagerM.cpp": tag = "platform(xbox stream/font)"
    elif "~" in n or "Dtor" in n or "scalar deleting" in n: tag = "ctor-dtor"
    elif re.search(r"::(\w+)::\1$", n): tag = "ctor-dtor"
    elif re.search(r"Debug|Tweak|debug", n + strs) or "GameTweaksMenu" in f: tag = "debug"
    elif sz < 40: tag = "getter/trivial"
    elif f in ("OSDManager.cpp", "OSD3dInterface.cpp", "OSDInGameScreen.cpp", "OSDScreens.cpp", "Speedo.cpp", "COSDMapRender.cpp", "GamePauseMenu.cpp", "GameMenuManager.cpp", "OSD3dInterfaceConfiguration.cpp"): tag = "gameplay-UI"
    elif "Editor" in f: tag = "editor-screen"
    elif sz < 120 and int(ncall) <= 4: tag = "getter/trivial"
    else: tag = "logic"
    rows.append((a, sz, f, n, tag, men, strs))
rows.sort(key=lambda r: -r[1])
c = Counter(); cb = Counter()
for r in rows: c[r[4].split("(")[0]] += 1; cb[r[4].split("(")[0]] += r[1]
hdr = open(os.path.join(root, "fe_cov_hdr.txt")).read().splitlines()
out = hdr + ["# unported by tag (count/bytes): " + "; ".join("%s %d/%d" % (k, c[k], cb[k]) for k in sorted(c, key=lambda k: -cb[k])),
             "# cols: addr size file name tag [noted = address appears in a comment, checked unported] strings"]
for a, sz, f, n, tag, men, strs in rows:
    out.append("0x%s\t%d\t%s\t%s\t%s\t%s\t%s" % (a, sz, f, n, tag, "noted" if men else "", strs[:70]))
open(os.path.join(root, "frontend_coverage.txt"), "w").write("\n".join(out) + "\n")
print(dict(c)); print(dict(cb))
