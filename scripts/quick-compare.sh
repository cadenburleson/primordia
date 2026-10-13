#!/bin/zsh
# A quick answer to "did this change help?" without running a planet from nothing.
#
#   scripts/quick-compare.sh <cycles> <saved planet>... [-- <creature> ...]
#
# Builds the engine as it is now, and runs it for <cycles> from each saved
# planet beside the engine it was last compared with (the "before" binary,
# kept from the previous call; the first call makes it). Any creatures named
# after -- are released into every run first, 32 founders of each. Runs go
# side by side, one core each. For every starter species it prints cells at
# the start and end, growth per hundred cycles and the commonest death, before
# and after. It refuses to compare two runs that released different numbers
# of grown bodies, because then they landed in different places and what a
# few animals ate says nothing.
#
#   scripts/quick-compare.sh keep     make the engine as it is now the "before"
#
# Saved planets are whatever `physical_batch --output` wrote. Keep them out of
# the repository: they are 50 to 250 MB each.
set -e
R=${0:a:h:h}; W=${QUICK_DIR:-$R/.quick}; mkdir -p $W
build() { (cd $R/engine && cargo build --release --bin physical_batch 2>&1 | grep -E "^error|-->" || true); cp $R/engine/target/release/physical_batch $1; }
if [ "$1" = keep ]; then build $W/before; echo "kept the engine as it is now as 'before'"; exit 0; fi
cycles=$1; shift; saves=(); release=()
while [ $# -gt 0 ] && [ "$1" != -- ]; do saves+=($1); shift; done
[ "$1" = -- ] && shift
for s in "$@"; do release+=(--steer "{\"kind\":\"seed\",\"species\":\"$s\",\"count\":32}"); done
[ ${#saves} -gt 0 ] || { echo "usage: quick-compare.sh <cycles> <saved planet>... [-- <creature> ...]"; exit 1; }
[ -x $W/before ] || { build $W/before; echo "no 'before' engine yet: made one from the code as it is. Change the engine and run this again."; exit 0; }
build $W/after
for save in $saves; do
  n=${save:t:r}
  for which in before after; do
    ($W/$which --input $save "${release[@]}" --cycles $cycles > $W/$n.$which.log 2>&1 || echo "FAILED $n $which: $(tail -1 $W/$n.$which.log)") &
  done
done
wait
python3 - $W ${saves:t:r} <<'EOF'
import re,sys
W=sys.argv[1]
def read(path):
    t=open(path).read()
    tally={m.group(1).strip():(int(m.group(2)),int(m.group(3)),int(m.group(4)),int(m.group(5)),float(m.group(6))) for m in re.finditer(r'^tally (.{14}) +(\d+) -> (\d+) +born +(\d+) died +(\d+) +([-+][\d.]+)%',t,re.M)}
    died={m.group(1).strip():m.group(2).split(',')[0] for m in re.finditer(r'^died (.{14}) +\d+: (.*)$',t,re.M)}
    released=re.findall(r'^(\d+ grown .*?) released',t,re.M)
    land=re.search(r'^land (\{.*\})',t,re.M); rungs=dict(re.findall(r'^ *(\w+) +stage (\d)',t,re.M))
    return tally,died,released,land.group(1) if land else '',rungs,'FAILED' in t or 'failed validation' in t
for n in sys.argv[2:]:
    b=read(f'{W}/{n}.before.log'); a=read(f'{W}/{n}.after.log')
    print(f"\n== {n} ==")
    if b[5] or a[5]: print("  a run failed; see", f'{W}/{n}.*.log'); continue
    if b[2]!=a[2]:
        print("  NOT COMPARABLE: the two engines released different bodies, so they landed in different places.")
        print("   before:", '; '.join(b[2])); print("   after: ", '; '.join(a[2])); continue
    print(f"  {'species':14}{'start':>7}{'end before':>11}{'after':>7}{'growth/100 before':>19}{'after':>9}   commonest death before / after")
    for s,(f,t,_,_,r) in sorted(b[0].items(), key=lambda kv:-kv[1][0]):
        if s not in a[0]: continue
        f2,t2,_,_,r2=a[0][s]; flag='  <<' if abs(t2-t)>max(5,0.15*max(t,t2)) else ''
        print(f"  {s:14}{f:>7}{t:>11}{t2:>7}{r:>18.1f}%{r2:>8.1f}%   {b[1].get(s,'-')} / {a[1].get(s,'-')}{flag}")
    changed=[f"{k} {b[4][k]}->{a[4][k]}" for k in a[4] if b[4].get(k)!=a[4][k]]
    print("  ladder steps that moved:", ', '.join(changed) or 'none')
    print("  land before:", b[3]); print("  land after: ", a[3])
EOF
