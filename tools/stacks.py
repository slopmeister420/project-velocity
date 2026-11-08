"""stacks.py <pid> [frames] [symbol dir]: suspend every thread of pid, print a symbolised stack per thread (dbghelp),
resume. For a hung hidden game: `tools/stacks.py <pid> 60 "<repo>/target/core/debug"` (the dir holding velocity.pdb)."""
import ctypes, sys
from ctypes import wintypes as W

pid = int(sys.argv[1])
depth = int(sys.argv[2]) if len(sys.argv) > 2 else 25
k32 = ctypes.WinDLL("kernel32", use_last_error=True)
dbg = ctypes.WinDLL("dbghelp", use_last_error=True)
k32.OpenProcess.restype = W.HANDLE
k32.OpenThread.restype = W.HANDLE
k32.CreateToolhelp32Snapshot.restype = W.HANDLE
hp = k32.OpenProcess(0x1F0FFF, False, pid)
dbg.SymSetOptions(0x2 | 0x10 | 0x200)  # deferred loads, load lines, undname
sp = sys.argv[3] if len(sys.argv) > 3 else None
if not dbg.SymInitializeW(W.HANDLE(hp), sp, True):
    sys.exit("SymInitialize failed %d" % ctypes.get_last_error())


class TE(ctypes.Structure):
    _fields_ = [("size", W.DWORD), ("usage", W.DWORD), ("tid", W.DWORD), ("pid", W.DWORD), ("base", W.LONG), ("delta", W.LONG), ("flags", W.DWORD)]


snap = k32.CreateToolhelp32Snapshot(4, 0)
te = TE()
te.size = ctypes.sizeof(TE)
tids = []
ok = k32.Thread32First(W.HANDLE(snap), ctypes.byref(te))
while ok:
    if te.pid == pid:
        tids.append(te.tid)
    ok = k32.Thread32Next(W.HANDLE(snap), ctypes.byref(te))

dbg.SymFunctionTableAccess64.restype = ctypes.c_void_p
dbg.SymGetModuleBase64.restype = ctypes.c_uint64
FTA = ctypes.cast(dbg.SymFunctionTableAccess64, ctypes.c_void_p)
GMB = ctypes.cast(dbg.SymGetModuleBase64, ctypes.c_void_p)
dbg.StackWalk64.argtypes = [W.DWORD, W.HANDLE, W.HANDLE, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]


def name(addr):
    buf = ctypes.create_string_buffer(88 + 512)
    ctypes.c_uint32.from_buffer(buf, 0).value = 88
    ctypes.c_uint32.from_buffer(buf, 80).value = 500
    disp = ctypes.c_uint64()
    if dbg.SymFromAddr(W.HANDLE(hp), ctypes.c_uint64(addr), ctypes.byref(disp), buf):
        return buf.raw[84:84 + 500].split(b"\0")[0].decode(errors="replace")
    return "?%x" % addr


for tid in tids:
    ht = k32.OpenThread(0x1FFFFF, False, tid)
    k32.SuspendThread(W.HANDLE(ht))
    raw = ctypes.create_string_buffer(1232 + 16)
    off = (-ctypes.addressof(raw)) % 16
    ctx = ctypes.addressof(raw) + off
    ctypes.c_uint32.from_address(ctx + 0x30).value = 0x10000B
    if not k32.GetThreadContext(W.HANDLE(ht), ctypes.c_void_p(ctx)):
        print("thread", tid, "no context", ctypes.get_last_error())
        k32.ResumeThread(W.HANDLE(ht))
        continue
    rip = ctypes.c_uint64.from_address(ctx + 0xF8).value
    rsp = ctypes.c_uint64.from_address(ctx + 0x98).value
    rbp = ctypes.c_uint64.from_address(ctx + 0xA0).value
    sf = ctypes.create_string_buffer(0x400)
    a = ctypes.addressof(sf)
    ctypes.c_uint64.from_address(a + 0).value = rip
    ctypes.c_uint32.from_address(a + 12).value = 3
    ctypes.c_uint64.from_address(a + 32).value = rbp
    ctypes.c_uint32.from_address(a + 44).value = 3
    ctypes.c_uint64.from_address(a + 48).value = rsp
    ctypes.c_uint32.from_address(a + 60).value = 3
    frames = []
    for _ in range(depth):
        if not dbg.StackWalk64(0x8664, hp, ht, a, ctx, None, FTA, GMB, None):
            break
        pc = ctypes.c_uint64.from_address(a).value
        if pc == 0:
            break
        frames.append(pc)
    k32.ResumeThread(W.HANDLE(ht))
    print("== thread", tid, flush=True)
    for f in frames:
        print("   ", name(f)[:180], flush=True)
