"""run a command (cargo, tests, scripts) at below-normal cpu priority so it never competes with the user's
foreground work. child processes (rustc, linker, test binaries) inherit the priority class.

usage: python tools/lowprio.py <cmd> [args...]     exit code = the command's exit code"""
import ctypes, ctypes.wintypes as w, subprocess, sys

BELOW_NORMAL_PRIORITY_CLASS = 0x4000


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    # lower our own class first so the child, and everything it spawns, starts below normal
    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    k32.GetCurrentProcess.restype = w.HANDLE
    k32.SetPriorityClass.argtypes = [w.HANDLE, w.DWORD]
    k32.SetPriorityClass(k32.GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS)
    sys.exit(subprocess.call(sys.argv[1:]))


main()
