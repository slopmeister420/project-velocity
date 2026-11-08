"""coverage sweep: every shipped race in FE/TRACKS.INI (race dir present in the game dir), dry and wet, run
headless through tools/hidden_run.py with the smoke driver (quick race to that race, follow the raceline for
--drive s, pause, quit). serial; silent (never passes --sound).

usage: .venv-ida/Scripts/python tools/sweep.py [--drive 60] [--timeout 420] [--only SUBSTR ...] [--failed]
       [--weather dry|wet|both] [--list] [--parallel [N]] [--direct] [--tsv FILE]
--parallel N (default 3) runs N games at once: each run has its own log, screenshot prefix and save dir
(research/sweep/saves/<key>); rows are merged into the results file as they finish. --direct sets SMOKE_DIRECT=1
(boot straight into the race, no menu presses). --tsv writes another results file than results.tsv.
writes research/sweep/results.tsv (one row per run, rows of earlier runs not re-run are kept) and
research/sweep/logs/<TRACK>_<TYPE>_<RACE>_<dry|wet>.log.
pass = exit 0 and the race was reached (a 'drove' or 'race finished' line), no panic, no smoke timeout.
--failed re-runs only the rows that are not 'pass' in the existing results.tsv."""
import concurrent.futures, os, re, subprocess, sys, threading, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GAME = os.environ.get("VELOCITY_GAME_DIR", "D:/games/xbox/games/project velocity debug")
OUT = os.path.join(ROOT, "research", "sweep")
LOGS = os.path.join(OUT, "logs")
TSV = os.path.join(OUT, "results.tsv")
PY = os.path.join(ROOT, ".venv-ida", "Scripts", "python.exe")
EXE = os.environ.get("VELOCITY_EXE", os.path.join(ROOT, "target", "debug", "velocity.exe"))
COLS = ["key", "area", "track", "type", "race", "category", "weather", "result", "exit", "secs", "panic", "tail"]


def find_dir(base, parts):
    """case-insensitive path lookup under base"""
    cur = base
    for p in parts:
        try:
            names = os.listdir(cur)
        except OSError:
            return None
        hit = [n for n in names if n.lower() == p.lower()]
        if not hit:
            return None
        cur = os.path.join(cur, hit[0])
    return cur


def races():
    text = open(os.path.join(GAME, "FE", "TRACKS.INI"), encoding="latin-1").read()
    out, cur = [], {}
    for line in text.splitlines():
        m = re.match(r'\s*(\w+)\s*:\s*"([^"]*)"', line)
        if not m:
            continue
        k, v = m.group(1).lower(), m.group(2)
        if k == "area" and cur:
            out.append(cur)
            cur = {}
        cur[k] = v
    if cur:
        out.append(cur)
    res = []
    for r in out:
        if find_dir(GAME, ["Tracks", r["track"], r["type"], r["race"]]):
            res.append(r)
    return res


def key_of(r, weather):
    return "%s_%s_%s_%s" % (r["track"].upper(), r["type"].upper(), r["race"].upper(), weather)


def classify(code, log):
    panic = ""
    m = re.search(r"panicked at ([^\n]*)\n([^\n]*)", log)
    if m:
        panic = (m.group(1) + " " + m.group(2)).strip()
    reached = "smoke-test: drove" in log or "smoke-test: race finished" in log
    if panic:
        res = "panic"
    elif code == 124:
        res = "killed"
    elif "smoke-test: timed out" in log:
        res = "timeout"
    elif "not reachable" in log or "not in the event database" in log:
        res = "unreachable"
    elif code != 0:
        res = "exit"
    elif not reached:
        res = "no_race"
    else:
        res = "pass"
    return res, panic


def clean(s):
    return s.replace("\t", " ").replace("\r", "").replace("\n", " | ")


def load():
    rows = {}
    if os.path.exists(TSV):
        lines = open(TSV, encoding="utf-8").read().splitlines()
        for l in lines[1:]:
            f = l.split("\t")
            if len(f) == len(COLS):
                rows[f[0]] = dict(zip(COLS, f))
    return rows


def save(rows):
    with open(TSV, "w", encoding="utf-8", newline="\n") as f:
        f.write("\t".join(COLS) + "\n")
        for k in sorted(rows):
            f.write("\t".join(rows[k][c] for c in COLS) + "\n")


def main():
    a = sys.argv[1:]
    global TSV
    drive, timeout, only, weathers, failed, listing = "60", 420.0, [], ["dry", "wet"], False, False
    jobs, direct = 1, False
    i = 0
    while i < len(a):
        if a[i] == "--parallel":
            jobs = int(a[i + 1]) if i + 1 < len(a) and a[i + 1].isdigit() else 3
            i += 2 if i + 1 < len(a) and a[i + 1].isdigit() else 1
        elif a[i] == "--direct":
            direct = True; i += 1
        elif a[i] == "--tsv":
            TSV = os.path.abspath(a[i + 1]); i += 2
        elif a[i] == "--drive":
            drive = a[i + 1]; i += 2
        elif a[i] == "--timeout":
            timeout = float(a[i + 1]); i += 2
        elif a[i] == "--only":
            only.append(a[i + 1].upper()); i += 2
        elif a[i] == "--weather":
            weathers = ["dry", "wet"] if a[i + 1] == "both" else [a[i + 1]]; i += 2
        elif a[i] == "--failed":
            failed = True; i += 1
        elif a[i] == "--list":
            listing = True; i += 1
        else:
            sys.exit(__doc__)
    os.makedirs(LOGS, exist_ok=True)
    rows = load()
    todo = []
    for r in races():
        for w in weathers:
            k = key_of(r, w)
            if only and not any(o in k.upper() for o in only):
                continue
            if failed and rows.get(k, {}).get("result") == "pass":
                continue
            todo.append((r, w, k))
    if listing:
        for _, _, k in todo:
            print(k)
        print(len(todo), "runs")
        return
    lock = threading.Lock()
    done = [0]

    def one(r, w, k):
        log_path = os.path.join(LOGS, k + ".log")
        path = "%s\\%s\\%s" % (r["track"], r["type"], r["race"])
        cmd = [PY, os.path.join(ROOT, "tools", "hidden_run.py"), "--timeout", str(timeout),
               "--env", "SMOKE_RACE_PATH=" + path, "--env", "SMOKE_DRIVE=" + drive,
               "--env", "SMOKE_FOLLOW_LINE=1", "--env", "SMOKE_PREFIX=" + ("sweep_" + k if jobs > 1 else "sweep"),
               "--env", "VELOCITY_GAME_DIR=" + GAME, "--env", "RUST_BACKTRACE=1"]
        if direct:
            cmd += ["--env", "SMOKE_DIRECT=1"]
        if jobs > 1:
            # own save dir per run (the smoke exe only makes one for SMOKE_CAREER)
            sd = os.path.join(OUT, "saves", k)
            os.makedirs(sd, exist_ok=True)
            cmd += ["--env", "VELOCITY_SAVE_DIR=" + sd]
        if w == "wet":
            cmd += ["--env", "SMOKE_WET=1"]
        cmd += ["--", EXE, "--smoke-test", "--game", GAME]
        t0 = time.time()
        with open(log_path, "w", encoding="utf-8", errors="replace") as lf:
            code = subprocess.call(cmd, stdout=lf, stderr=subprocess.STDOUT, cwd=ROOT)
        secs = time.time() - t0
        log = re.sub(r"\x1b\[[0-9;]*m", "", open(log_path, encoding="utf-8", errors="replace").read())
        res, panic = classify(code, log)
        body = log.split("stack backtrace:")[0]
        tail = [re.sub(r"^\S+Z\s+", "", l) for l in body.splitlines() if l.strip()][-6:]
        # first game frames of the panic's backtrace (crates/...), so the owner is visible in the row
        frames = re.findall(r"\n\s+\d+: ((?:velocity|visos)\w*::[^\n]+)", log.split("stack backtrace:")[1]) if panic and "stack backtrace:" in log else []
        if frames:
            panic += " @ " + " < ".join(f.strip() for f in frames[:3])
        with lock:
            rows[k] = {"key": k, "area": r.get("area", ""), "track": r["track"], "type": r["type"], "race": r["race"],
                       "category": r.get("category", ""), "weather": w, "result": res, "exit": str(code),
                       "secs": "%.0f" % secs, "panic": clean(panic)[:400], "tail": clean(" || ".join(tail))[:1200]}
            save(rows)
            done[0] += 1
            print("%3d/%d %-40s %-8s %4.0fs %s" % (done[0], len(todo), k, res, secs, panic[:100]), flush=True)

    t_all = time.time()
    if jobs <= 1:
        for r, w, k in todo:
            one(r, w, k)
    else:
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
            list(ex.map(lambda a: one(*a), todo))
    print("%d runs in %.0fs (jobs %d%s)" % (len(todo), time.time() - t_all, jobs, ", direct" if direct else ""))


main()
