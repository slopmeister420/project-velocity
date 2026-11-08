# frontend agent: print the C strings at the given addresses (fe_peek_args.txt)
import idc
for a in open(r"D:\projects\project poland rewrite\tools\ida\fe_peek_args.txt").read().split():
    print(a, idc.get_strlit_contents(int(a, 16)))
