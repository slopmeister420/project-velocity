import idc, idautils
# callers of the GameManager movie entry points and the VsMovieStream API
base = r"D:\projects\project poland rewrite\research"
out = []
for a in (0xBE9F0, 0xBEBB0, 0x2F4C90, 0x2F4AD0, 0x2F4BD0, 0x2F4C00, 0x2F4C70, 0x2F5E60, 0xC0050, 0xBEFD0):
    refs = ["%X(%s)" % (r, idc.get_func_name(r)) for r in idautils.CodeRefsTo(a, 0)]
    out.append("%X %s: %s" % (a, idc.get_func_name(a), " ".join(refs)))
open(base + r"\fe_movie_xrefs.txt", "w").write("\n".join(out))
