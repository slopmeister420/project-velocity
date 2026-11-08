"""run a test exe on a separate, invisible windows desktop so it can never show up or take focus on the
user's desktop. never touches windows on the default desktop.

usage: python tools/hidden_run.py [--timeout S] [--env K=V ...] [--sound] -- <exe> [args...]
exit code = the program's exit code (or 124 on timeout, after killing it).
sets VELOCITY_AUDIO_OUT=null (mix as normal, nothing to the speakers); --sound opts out.
sets VELOCITY_HIDDEN=1 (no SDL, no keyboard, no raw input; scripted input only).
throwaway profile: unless the caller passes --env APPDATA=..., APPDATA (and VELOCITY_SAVE_DIR) point at a fresh
temp dir seeded from research/golden/profile_seed, deleted after the run (--keep-profile keeps it), so a test run
never writes the user's real %APPDATA%\\Velocity (options.ini, saves)."""
import ctypes, ctypes.wintypes as w, os, shutil, sys, tempfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SEED = os.path.join(ROOT, "research", "golden", "profile_seed", "Velocity")

k32 = ctypes.WinDLL("kernel32", use_last_error=True)
u32 = ctypes.WinDLL("user32", use_last_error=True)
DESK = "velocitytest"


class SI(ctypes.Structure):
    _fields_ = [("cb", w.DWORD), ("r", w.LPWSTR), ("desk", w.LPWSTR), ("title", w.LPWSTR)] + \
               [(n, w.DWORD) for n in "x y xs ys xc yc fill flags".split()] + \
               [("show", w.WORD), ("r2", w.WORD), ("r3", ctypes.c_void_p),
                ("i", w.HANDLE), ("o", w.HANDLE), ("e", w.HANDLE)]


class PI(ctypes.Structure):
    _fields_ = [("hp", w.HANDLE), ("ht", w.HANDLE), ("pid", w.DWORD), ("tid", w.DWORD)]


def main():
    a = sys.argv[1:]
    if "--" not in a:
        sys.exit(__doc__)
    opts, cmd = a[:a.index("--")], a[a.index("--") + 1:]
    timeout, env = 300.0, dict(os.environ)
    env["VELOCITY_AUDIO_OUT"] = "null"
    # input-inert: the exe skips SDL (no pad raw input / HIDAPI / rumble) and the keyboard, scripted input only
    env["VELOCITY_HIDDEN"] = "1"
    i, given, keep = 0, set(), False
    while i < len(opts):
        if opts[i] == "--timeout":
            timeout = float(opts[i + 1]); i += 2
        elif opts[i] == "--sound":
            env.pop("VELOCITY_AUDIO_OUT", None); i += 1
        elif opts[i] == "--keep-profile":
            keep = True; i += 1
        elif opts[i] == "--env":
            k, v = opts[i + 1].split("=", 1); env[k] = v; given.add(k.upper()); i += 2
        else:
            sys.exit("unknown option " + opts[i])
    prof = None
    if "APPDATA" not in given:
        prof = tempfile.mkdtemp(prefix="velocity_profile_")
        vel = os.path.join(prof, "Velocity")
        shutil.copytree(SEED, vel) if os.path.isdir(SEED) else os.makedirs(vel)
        env["APPDATA"] = prof
        if "VELOCITY_SAVE_DIR" not in given:
            env["VELOCITY_SAVE_DIR"] = os.path.join(vel, "Saves")
        if keep:
            print("hidden_run: profile " + prof, file=sys.stderr)
    try:
        run(env, cmd, timeout)
    finally:
        if prof and not keep:
            shutil.rmtree(prof, ignore_errors=True)


def run(env, cmd, timeout):
    u32.CreateDesktopW.restype = w.HANDLE
    desk = u32.CreateDesktopW(DESK, None, None, 0, 0x10000000, None)  # GENERIC_ALL
    if not desk:
        # never fall back to the user's desktop
        sys.exit("CreateDesktop failed %d" % ctypes.get_last_error())
    si = SI(); si.cb = ctypes.sizeof(SI); si.desk = DESK
    pi = PI()
    cmd[0] = os.path.abspath(cmd[0])
    line = ctypes.create_unicode_buffer(" ".join('"%s"' % c if " " in c else c for c in cmd))
    block = ctypes.create_unicode_buffer("".join("%s=%s\0" % kv for kv in env.items()) + "\0")
    CREATE_UNICODE_ENVIRONMENT = 0x400
    BELOW_NORMAL_PRIORITY_CLASS = 0x4000  # test games never compete with the user's foreground work
    if not k32.CreateProcessW(None, line, None, None, True, CREATE_UNICODE_ENVIRONMENT | BELOW_NORMAL_PRIORITY_CLASS,
                              block, os.getcwd(),
                              ctypes.byref(si), ctypes.byref(pi)):
        sys.exit("CreateProcess failed %d" % ctypes.get_last_error())
    t0, code = time.time(), w.DWORD()
    while time.time() - t0 < timeout:
        k32.GetExitCodeProcess(pi.hp, ctypes.byref(code))
        if code.value != 259:  # STILL_ACTIVE
            sys.exit(code.value)
        time.sleep(0.5)
    k32.TerminateProcess(pi.hp, 124)
    print("hidden_run: killed after %.0f s" % timeout, file=sys.stderr)
    sys.exit(124)


main()
