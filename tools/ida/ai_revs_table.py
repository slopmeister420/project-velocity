# revs curve name table used by VsAIWorld::SetupStartlineRevs (ai agent)
import ida_bytes
n = ida_bytes.get_dword(0x684E80)
print("count", n)
for i in range(min(n, 8)):
    p = ida_bytes.get_dword(0x684E84 + 4 * i)
    print(i, hex(p), ida_bytes.get_strlit_contents(p, -1, 0))
