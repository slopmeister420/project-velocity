#!/bin/sh
# physics agent: regenerate every translated tr.rs from research/phys_dump (address list research/_tr_addrs.txt)
cd "$(dirname "$0")/.."
python tools/phys_c2rs4.py "$(cygpath -w "$PWD/crates/velocity_physics/src")" $(sort -u research/_tr_addrs.txt)
