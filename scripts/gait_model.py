"""A paper model of the three things docs/ANIMAL_BODIES_PLAN.md says a body needs to travel.

Nothing here touches the engine. It is plain arithmetic, runs in a few seconds on one core, and
answers before any engine change: how far a jointed body goes per beat under drag that differs by
direction, and whether the engine's own nerve equations can carry a wave down two sides in turn.

    python3 -I scripts/gait_model.py
"""
import math

# ---------------------------------------------------------------------------------------------
# Part A. A chain of stiff links in thick water (drag proportional to speed, no inertia).
# Each link resists moving across itself `rho` times as hard as along itself. The joint angles
# are given; the water decides where the body goes: at every instant the total force and the
# total turning force on the body are zero.
# ---------------------------------------------------------------------------------------------
SAMPLES = 4  # points along each link at which the water's push is summed


def shape(angles, n):
    """Points and link directions in the head link's own frame, for the given joint angles."""
    length = 1.0 / n
    points, x, y, heading = [], 0.0, 0.0, 0.0
    for i in range(n):
        if i > 0:
            heading += angles[i - 1]
        c, s = math.cos(heading), math.sin(heading)
        for k in range(SAMPLES):
            f = (k + 0.5) / SAMPLES * length
            points.append((x + c * f, y + s * f, c, s, i))
        x, y = x + c * length, y + s * length
    return points


def swim(n, gait, rho, beats=3, steps=240, fin=None):
    """Net travel in body lengths per beat, and net turn per beat, of an n-link chain."""
    fin = fin or {}
    px = py = theta = 0.0
    dt = 1.0 / steps
    start = end = None
    for step in range(beats * steps + 1):
        t = step * dt
        if step % steps == 0:
            at = shape(gait(t), n)
            cx = sum(p[0] for p in at) / len(at)
            cy = sum(p[1] for p in at) / len(at)
            c, s = math.cos(theta), math.sin(theta)
            centre = (px + c * cx - s * cy, py + s * cx + c * cy, theta)
            if step == steps:
                start = centre
            if step == beats * steps:
                end = centre
        # The body's speed is worked out at the middle of each step, which keeps the sum honest.
        now = shape(gait(t + dt / 2), n)
        before, after = shape(gait(t), n), shape(gait(t + dt), n)
        a = [[0.0] * 3 for _ in range(3)]
        b = [0.0] * 3
        for (x, y, tx, ty, link), q0, q1 in zip(now, before, after):
            across = 1.0 + fin.get(link, 0.0)
            along = 1.0 / rho
            # The water's hold on this point: `across` every way, less along the link.
            m11 = across - (across - along) * tx * tx
            m12 = -(across - along) * tx * ty
            m22 = across - (across - along) * ty * ty
            vx, vy = (q1[0] - q0[0]) / dt, (q1[1] - q0[1]) / dt
            wx, wy = m11 * -y + m12 * x, m12 * -y + m22 * x
            gx, gy = m11 * vx + m12 * vy, m12 * vx + m22 * vy
            a[0][0] += m11; a[0][1] += m12; a[0][2] += wx; b[0] -= gx
            a[1][0] += m12; a[1][1] += m22; a[1][2] += wy; b[1] -= gy
            a[2][0] += x * m12 - y * m11
            a[2][1] += x * m22 - y * m12
            a[2][2] += x * wy - y * wx
            b[2] -= x * gy - y * gx
        ux, uy, spin = solve3(a, b)
        c, s = math.cos(theta + spin * dt / 2), math.sin(theta + spin * dt / 2)
        px += (c * ux - s * uy) * dt
        py += (s * ux + c * uy) * dt
        theta += spin * dt
    per = beats - 1
    return math.hypot(end[0] - start[0], end[1] - start[1]) / per, (end[2] - start[2]) / per


def solve3(a, b):
    m = [row[:] + [v] for row, v in zip(a, b)]
    for i in range(3):
        pivot = max(range(i, 3), key=lambda r: abs(m[r][i]))
        m[i], m[pivot] = m[pivot], m[i]
        for r in range(3):
            if r != i:
                f = m[r][i] / m[i][i]
                m[r] = [x - f * y for x, y in zip(m[r], m[i])]
    return [m[i][3] / m[i][i] for i in range(3)]


def wave(n, swing, waves=1.0):
    """Every joint swings `swing` radians each way, each later than the one ahead."""
    return lambda t: [swing * math.sin(2 * math.pi * (waves * j / n - t)) for j in range(1, n)]


def together(n, swing):
    """Every joint swings at the same moment: the body curls and uncurls."""
    return lambda t: [swing * math.sin(2 * math.pi * t) for _ in range(1, n)]


def theory(n, swing, rho, waves=1.0):
    """Gray and Hancock's balance for a smooth wave of the same curvature."""
    k = 2 * math.pi * waves
    height = swing * n / (k * k)
    beta = (height * k) ** 2 / 2
    return beta * (rho - 1) / (1 + rho * beta) / waves, height


def part_a():
    print("A. Travel of a jointed body, in body lengths per beat")
    print("   (swing = how far each joint bends each way; height = the wave's height, body lengths)\n")
    print("   Checks of the model itself:")
    print(f"     same drag every way (rho 1), 9 links, wave      {swim(9, wave(9, 0.45), 1.0)[0]:.4f}   (must be 0)")
    print(f"     rho 2, 9 links, every joint at the same moment  {swim(9, together(9, 0.2), 2.0)[0]:.4f}   (must be 0)")
    got = swim(40, wave(40, 0.02), 2.0)[0]
    print(f"     rho 2, 40 links, one small wave                 {got:.4f}   (smooth-wave formula {theory(40, 0.02, 2.0)[0]:.4f})")
    got = swim(120, wave(120, 0.06, 3.0), 2.0)[0]
    print(f"     rho 2, 120 links, three small waves             {got:.4f}   (smooth-wave formula {theory(120, 0.06, 2.0, 3.0)[0]:.4f})\n")
    print("   links  swing  height   rho 1.4   rho 2    rho 2 + tail fin x4   formula at rho 2")
    for n in (5, 9, 13, 18):
        for swing in (0.22, 0.45, 0.7):
            cells = [swim(n, wave(n, swing), rho)[0] for rho in (1.4, 2.0)]
            finned = swim(n, wave(n, swing), 2.0, fin={n - 1: 3.0, n - 2: 3.0})[0]
            formula, height = theory(n, swing, 2.0)
            print(f"   {n:5d}  {swing:5.2f}  {height:6.3f}   {cells[0]:7.3f}  {cells[1]:6.3f}   {finned:12.3f}          {formula:8.3f}")
    print("\n   How late each joint may be (9 links, swing 0.45, rho 2): waves on the body -> travel")
    print("    ", "  ".join(f"{w:.2f}: {swim(9, wave(9, 0.45, w), 2.0)[0]:.3f}" for w in (0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0)))
    purcell = lambda t: [0.9 * math.sin(2 * math.pi * t), 0.9 * math.sin(2 * math.pi * t - math.pi / 2)]
    print(f"\n   Purcell's swimmer (3 links, two hinges a quarter-beat apart, swing 0.9, rho 2): {swim(3, purcell, 2.0)[0]:.3f}")


# ---------------------------------------------------------------------------------------------
# Part B. The engine's own nerve equations (physiology.rs, excitable_signals), on two lines of
# nerve cells, one down each side. Only the head cell of each line is driven. Ties along a line
# pass firing on; the two head cells hold each other down.
# ---------------------------------------------------------------------------------------------
NERVE_STEPS = 24
STEP = 0.25  # DT / NERVE_STEPS * NERVE_PACE


def ladder(n, drive, tie, cross, ticks=400, cross_all=False, gates=None, sampled=False, across=0.0):
    """`drive` is one number (the head cells only) or a list (every cell, head first).
    `gates` is how much channel each cell has, head first: fewer channels, a slower cell."""
    drives = drive if isinstance(drive, list) else [drive] + [0.0] * (n - 1)
    gates = gates or [1.0] * n
    u = [[0.0] * n, [0.0] * n]
    v = [[0.0] * n, [0.0] * n]
    u[0][0] = 0.5  # one side starts first
    spikes = [[[] for _ in range(n)] for _ in range(2)]
    seen = [[[] for _ in range(n)] for _ in range(2)]
    for tick in range(ticks):
        # `sampled`: the hold across is what the engine's junctions give (development.rs,
        # neural_contacts): read from the other cell once a tick and held through the tick.
        seen_u = [row[:] for row in u]
        for sub in range(NERVE_STEPS):
            new_u = [row[:] for row in u]
            new_v = [row[:] for row in v]
            for side in range(2):
                for i in range(n):
                    x, y = u[side][i], v[side][i]
                    coupling = 0.0
                    if i > 0:
                        coupling += (u[side][i - 1] - x) * tie
                    if i < n - 1:
                        coupling += (u[side][i + 1] - x) * tie
                    if i == 0 or cross_all:
                        coupling -= cross * (seen_u if sampled else u)[1 - side][i]
                    # `across`: a plain tie between the two cells of a pair, which passes firing
                    # on as the ties along a line do. The engine gives every tie between two
                    # nerve cells this, wanted or not.
                    coupling += (u[1 - side][i] - x) * across
                    du = 3 * x * (1 - x) * (x - 0.18) - y + drives[i] + coupling
                    dv = 0.12 * (x - y * 0.8)
                    nx = min(1.0, max(0.0, x + STEP * gates[i] * du))
                    new_u[side][i] = nx
                    new_v[side][i] = min(1.0, max(0.0, y + STEP * gates[i] * dv))
                    if x < 0.65 <= nx:
                        spikes[side][i].append(tick + sub / NERVE_STEPS)
            u, v = new_u, new_v
        for side in range(2):
            for i in range(n):
                # What a muscle is told, once a tick, as motor_signals reads it.
                seen[side][i].append(min(1.0, max(0.0, (u[side][i] - 0.3) * 5)))
    return spikes, seen


def measure(spikes, seen, n):
    head = [t for t in spikes[0][0] if t > 100]
    if len(head) < 4:
        return None
    period = (head[-1] - head[0]) / (len(head) - 1)
    reached = all(len([t for t in spikes[s][n - 1] if t > 100]) >= len(head) - 2 for s in range(2))
    def phase(times, ref):
        late = [((t - max(r for r in ref if r <= t)) / period) for t in times if t > 100 and any(r <= t for r in ref)]
        return sum(late) / len(late) if late else float("nan")
    lag = phase(spikes[0][n - 1], spikes[0][0]) if reached else float("nan")
    # A full body's delay may pass one beat: count whole beats by the time to first arrival.
    first_head, first_tail = spikes[0][0][0], (spikes[0][n - 1][0] if spikes[0][n - 1] else float("nan"))
    travel = (first_tail - first_head) / period
    sides = phase(spikes[1][0], spikes[0][0])
    caught = sum(1 for s in seen[0][0][100:] if s > 0.5) / max(1, len(head))
    return period, reached, travel, sides, caught


def wave_on(spikes, n, after=150):
    """Beat length, whether every cell keeps the head's beat, how many waves lie on the body,
    and how far the right side fires after the left, for a ladder in which every cell fires."""
    head = [t for t in spikes[0][0] if t > after]
    if len(head) < 6:
        return None
    period = (head[-1] - head[0]) / (len(head) - 1)
    locked = all(abs(len([t for t in spikes[s][i] if t > after]) - len(head)) <= 1 for s in range(2) for i in range(n))

    def late(times, ref):
        got = [(t - max(r for r in ref if r <= t)) / period % 1.0 for t in times if t > after and ref and ref[0] <= t]
        # Average on the circle, so that 0.98 and 0.02 are neighbours.
        x = sum(math.cos(2 * math.pi * g) for g in got)
        y = sum(math.sin(2 * math.pi * g) for g in got)
        return math.atan2(y, x) / (2 * math.pi) % 1.0 if got else float("nan")

    total, steps = 0.0, []
    for i in range(1, n):
        step = (late(spikes[0][i], spikes[0][i - 1]) + 0.5) % 1.0 - 0.5
        steps.append(step)
        total += step
    sides = sum(late(spikes[1][i], spikes[0][i]) for i in range(n)) / n
    return period, locked, total, min(steps), max(steps), sides


def part_b():
    n = 9
    print("\nB. The engine's own nerve equations on two lines of 9 nerve cells, one down each side")
    print("\n   B1. Only the head cells driven: does firing pass down the line? Firings per cell, head first")
    for drive, tie in ((0.35, 0.1), (0.35, 0.3), (0.35, 0.6), (0.35, 1.2)):
        spikes, _ = ladder(n, drive, tie, 0.3, ticks=200)
        print(f"     drive {drive}, tie {tie:<4}: {[len(x) for x in spikes[0]]}")
    print("\n   B2. Every cell driven alike (as in the engine now), each a little slower than the one")
    print("       ahead (fewer channels toward the tail), ties along each side, and each cell")
    print("       holding down the one across from it.")
    print("       beat = ticks between head firings; in step = every cell keeps the head's beat;")
    print("       waves = waves lying on the body, head to tail (the lamprey has 1.0; 0.5 to 0.75")
    print("       travels best in part A); sides = right after left, in beats (0.5 = taking turns)\n")
    print("   links  drive  channels at tail   tie    hold   beat   in step   waves   sides")
    for links, drive, fall, tie, cross in (
        (9, 0.35, 0.0, 0.05, 0.0), (9, 0.35, 0.0, 0.05, 0.1), (9, 0.35, 0.1, 0.05, 0.0),
        (9, 0.35, 0.05, 0.05, 0.1), (9, 0.35, 0.1, 0.05, 0.1), (9, 0.35, 0.1, 0.1, 0.1), (9, 0.35, 0.1, 0.05, 0.2),
        (9, 0.25, 0.1, 0.05, 0.1), (9, 0.25, 0.1, 0.1, 0.2), (9, 0.35, 0.2, 0.05, 0.1),
        (5, 0.35, 0.1, 0.05, 0.1), (13, 0.35, 0.1, 0.05, 0.1), (13, 0.35, 0.05, 0.05, 0.1),
    ):
        gates = [1 - fall * i / (links - 1) for i in range(links)]
        spikes, _ = ladder(links, [drive] * links, tie, cross, ticks=500, cross_all=True, gates=gates)
        got = wave_on(spikes, links)
        if got is None:
            print(f"   {links:5d}  {drive:5.2f}  {1 - fall:8.2f}          {tie:5.2f}  {cross:5.2f}   no steady beat")
            continue
        period, locked, total, _, _, sides = got
        print(f"   {links:5d}  {drive:5.2f}  {1 - fall:8.2f}          {tie:5.2f}  {cross:5.2f}  {period:5.2f}   {'yes' if locked else 'no ':3s}      {total:5.2f}   {sides:5.2f}")


# ---------------------------------------------------------------------------------------------
# Part C. The whole swimmer, with the engine's own numbers (physical.rs, mechanics): cells with
# mass, ties that are damped springs, joints with a capped twist, the water's hold on the body as
# a whole and on each cell against its neighbours, and the nerves of part B driving the joints.
# Flat water, seen from above. What is new beside the engine is marked NEW.
# ---------------------------------------------------------------------------------------------
TICK = 0.25                      # DT
SUB = TICK / 20                  # a mechanical substep
BODY_DRAG = 6 * math.pi * 3.0 * ((0.9 - 0.5) / 3 * (64 / 40) ** 2 / TICK) * 0.08
RADIUS, REST = 0.22, 0.44
MUSCLE_PULL, MUSCLE_SPRING, SHORTEN = 300.0, 0.6, 0.22
JOINT_STRENGTH, JOINT_TORQUE, JOINT_PUSH = 30.0, 1.0, 20.0
SWING = 0.45                     # how far a joint's muscle turns it, each way
STOP, STOP_STIFF = 0.9, 60.0     # NEW: where bone meets bone, and how hard
SHIELD = 0.25                    # NEW: the cover each tie gives a cell along itself
FIN_SPAN = 3.0                   # NEW: how much wider a fin makes a cell, across its line


def swimmer(n=9, shaped=True, fins="none", fall=0.1, hold=0.1, nerve_tie=0.05, bone_tie=600.0,
            mineral=0.5, beats=30, drive=0.35, flank_pull=True, brace=0.0, torque=JOINT_TORQUE,
            swing_to=SWING, soft=None, ease=1.0, spring=0.0, water="engine", fair=False, sine=None, flanks=True, stiff=None, small=1.0, cord=False, rib=None, sampled=False, pace=1.0, square=0.0, one_line=None, trace=False):
    """A stiff middle line of n bone cells, a nerve-and-muscle cell on each flank of each.
    Returns body lengths a beat, the widest joint swing, and the longest any tie was stretched."""
    # Cells: 0..n-1 the middle, n..2n-1 the left flank (+z), 2n..3n-1 the right flank (-z).
    pos = [[i * REST, 0.0] for i in range(n)]
    if flanks:
        pos += [[i * REST, REST] for i in range(n)] + [[i * REST, -REST] for i in range(n)]
    vel = [[0.0, 0.0] for _ in pos]
    # `small` makes the flank cells that much narrower than a bone cell: light passengers.
    mass = [0.27 / (1 - mineral)] * n + ([0.27 * small ** 3] * (2 * n) if flanks else [])
    wide = [RADIUS] * n + ([RADIUS * small] * (2 * n) if flanks else [])
    off = RADIUS * small if cord else RADIUS * (1 + small)
    if flanks:
        # `cord`: the two lines of nerve cells ride side by side on top of the bones, as a nerve
        # cord does, each touching the other. Seen from above they overlie the bones.
        for i in range(n):
            pos[n + i][1] = off
            pos[2 * n + i][1] = -off
    stiff = mineral * JOINT_STRENGTH if stiff is None else stiff
    muscle = 0.05 / (0.015 * 0.25 + 0.05)          # activity of a fully built protein
    fin = [0.0] * len(pos)
    for side in ((0,) if fins.startswith("spine") or not flanks else (0, n, 2 * n) if fins == "both" else (n, 2 * n)):
        for i in range(n):
            if fins in ("all", "spine", "both") or (fins in ("tail", "spine tail") and i >= n - 2):
                fin[side + i] = muscle
    # Ties: (a, b, stiffness of the spring at rest, kind)
    ties = []
    for i in range(n - 1):
        ties.append([i, i + 1, 1 + mineral * 60 + bone_tie, "bone", REST])
        if brace:
            # A brace from each flank cell to the bone cells ahead and behind: with these the
            # three lines cannot slide past one another.
            for side in ((n, 2 * n) if flanks else ()):
                ties.append([side + i, i + 1, brace, "brace", REST * math.sqrt(2)])
                ties.append([side + i + 1, i, brace, "brace", REST * math.sqrt(2)])
        for side in ((n, 2 * n) if flanks else ()):
            ties.append([side + i, side + i + 1, soft or 1 + muscle * MUSCLE_SPRING * 60, "flank", REST])
    for i in range(n):
        for side in ((n, 2 * n) if flanks else ()):
            ties.append([i, side + i, rib or soft or 1 + 0.5 * muscle * MUSCLE_SPRING * 60, "rib", off])
        if flanks and cord:
            ties.append([n + i, 2 * n + i, soft or 3.0, "across", 2 * off])
    activation = [0.0] * len(ties)
    tied = {(min(t[0], t[1]), max(t[0], t[1])) for t in ties if t[3] != "brace"}
    target = [0.0] * n
    # Nerves on the flank cells only. `pace` is how much channel they have in all: with less,
    # every nerve cell runs slower and the whole beat is longer.
    gates = [pace * (1 - fall * i / (n - 1)) for i in range(n)]
    u = [[0.0] * n, [0.0] * n]
    v = [[0.0] * n, [0.0] * n]
    u[0][0] = 0.5
    total = sum(mass)
    single = [BODY_DRAG * r * SUB / m for r, m in zip(wide, mass)]
    length = (n - 1) * REST
    head_fires, marks, swing, stretch = [], [], 0.0, 0.0
    work, work_marks = 0.0, []      # what the joints' muscle does on the body, and its tally at each beat
    seen_bends = []                 # every joint's bend at the end of each tick

    def centre():
        return (sum(m * p[0] for m, p in zip(mass, pos)) / total, sum(m * p[1] for m, p in zip(mass, pos)) / total)

    tick = 0
    while len(head_fires) < beats + 1 and tick < beats * (12 / pace if not sine else sine[0] + 1):
        # Nerves, as in part B.
        held = [r[:] for r in u]
        for _ in range(NERVE_STEPS):
            nu, nv = [r[:] for r in u], [r[:] for r in v]
            for s in range(2):
                for i in range(n):
                    x, y = u[s][i], v[s][i]
                    c = -hold * (held if sampled else u)[1 - s][i]
                    if i > 0:
                        c += (u[s][i - 1] - x) * nerve_tie
                    if i < n - 1:
                        c += (u[s][i + 1] - x) * nerve_tie
                    du = 3 * x * (1 - x) * (x - 0.18) - y + drive + c
                    nx = min(1.0, max(0.0, x + STEP * gates[i] * du))
                    nu[s][i] = nx
                    nv[s][i] = min(1.0, max(0.0, y + STEP * gates[i] * 0.12 * (x - y * 0.8)))
                    if not sine and s == 0 and i == 0 and x < 0.65 <= nx:
                        head_fires.append(tick)
                        marks.append(centre())
                        work_marks.append(work)
            u, v = nu, nv
        firing = [[min(1.0, max(0.0, (u[s][i] - 0.3) * 5)) for i in range(n)] for s in range(2)]
        # Muscle in the ties, and the joints' aims, once a tick.
        for k, (a, b, _, kind, _) in enumerate(ties):
            if kind in ("bone", "brace", "across") or not flank_pull:
                continue
            if kind == "flank":
                s, i = (0, a - n) if a < 2 * n else (1, a - 2 * n)
                want = muscle * (firing[s][i] + firing[s][i + 1]) / 2
            else:
                s, i = (0, b - n) if b < 2 * n else (1, b - 2 * n)
                want = 0.5 * muscle * firing[s][i] / 2
            activation[k] += (want - activation[k]) * (0.9 if want > activation[k] else 0.4)
        for i in range(1, n - 1):
            pull = muscle * (firing[0][i] - firing[1][i])        # NEW: a joint is pulled from its two sides
            if one_line is not None:
                # One nerve cell to a segment, standing for both of its sides: the joint is
                # pulled one way while the cell fires and the other way while it recovers.
                after = min(1.0, max(0.0, (v[0][i] - one_line) * 5)) * (1 - firing[0][i])
                pull = muscle * (firing[0][i] - after)
            # The beat is brought up over the first beats, as a body that grows into its stroke.
            aim = max(-STOP, min(STOP, pull * swing_to * min(1.0, tick / 40)))
            if sine:
                # The joints' aims given by hand as a smooth wave: (ticks a beat, waves on the body).
                aim = swing_to * min(1.0, tick / 40) * math.sin(2 * math.pi * (sine[1] * i / n - tick / sine[0]))
            target[i] += (aim - target[i]) * ease      # a muscle takes time to pull a joint round
        # Each cell's line, for the water's hold by shape.
        if shaped:
            lines = [[0.0, 0.0, 0.0] for _ in pos]   # sums of dx*dx, dx*dz, dz*dz over a cell's ties
            around = [[] for _ in pos]
            for a, b, _, kind, _ in ties:
                dx, dz = pos[b][0] - pos[a][0], pos[b][1] - pos[a][1]
                d = math.hypot(dx, dz) or 1e-6
                dx, dz = dx / d, dz / d
                for i in (a, b):
                    # A bone's line is the line of its bone ties.
                    if not (i < n and kind != "bone"):
                        lines[i][0] += dx * dx; lines[i][1] += dx * dz; lines[i][2] += dz * dz
                    # A narrow passenger shades a bone little.
                    if i < n and kind != "bone" and small != 1.0:
                        continue
                    around[i].append((dx, dz))
            axes = []
            for xx, xz, zz in lines:
                angle = 0.5 * math.atan2(2 * xz, xx - zz)
                axes.append((math.cos(angle), math.sin(angle)))
        cx, cz = centre()
        reach = max(math.hypot(p[0] - cx, p[1] - cz) for p in pos) + RADIUS
        whole = BODY_DRAG * reach * SUB / total
        for _ in range(20):
            force = [[0.0, 0.0] for _ in pos]
            for k, (a, b, tie_stiff, kind, rest) in enumerate(ties):
                dx, dz = pos[b][0] - pos[a][0], pos[b][1] - pos[a][1]
                d = math.hypot(dx, dz) or 1e-6
                kk = tie_stiff + MUSCLE_PULL * activation[k]
                aim = rest * (1 - SHORTEN * activation[k])
                parting = ((vel[b][0] - vel[a][0]) * dx + (vel[b][1] - vel[a][1]) * dz) / d
                slight = min(min(mass[a], mass[b]) / 0.25, 1.0)      # a tie between small cells is a small tie
                pull = (min(d - aim, aim * 0.5) * kk + parting * 3.0 * math.sqrt(max(kk / 40, 1))) * slight / d
                force[a][0] += dx * pull; force[a][1] += dz * pull
                force[b][0] -= dx * pull; force[b][1] -= dz * pull
                room = (wide[a] + wide[b]) * (1 - SHORTEN * activation[k])
                if kind not in ("brace", "rib" if cord else "") and d < room:                         # two cells are stopped by their own bulk
                    push = (room - d) * 4 / d
                    force[a][0] -= dx * push; force[a][1] -= dz * push
                    force[b][0] += dx * push; force[b][1] += dz * push
                stretch = max(stretch, d / rest)
            # Cells that are not tied are still stopped by each other's bulk.
            for i in range(len(pos)):
                for j in range(i + 1, len(pos)):
                    if (i, j) in tied or (cord and i < n <= j):
                        continue
                    dx, dz = pos[j][0] - pos[i][0], pos[j][1] - pos[i][1]
                    reach2 = wide[i] + wide[j]
                    if abs(dx) < reach2 and abs(dz) < reach2:
                        d = math.hypot(dx, dz) or 1e-6
                        if d < reach2:
                            push = (reach2 - d) * 4 / d
                            force[i][0] -= dx * push; force[i][1] -= dz * push
                            force[j][0] += dx * push; force[j][1] += dz * push
            # NEW (`square`): a flank cell is held square to the rod where it is fastened, as the
            # sheet between two muscle blocks is fastened to the rod. Square to the line through
            # the bones either side, so that it does nothing to straighten the rod itself. A plain
            # tie holds only a length, and blocks held by plain ties slide along the rod.
            if square and flanks:
                for i in range(n):
                    lo, hi = max(i - 1, 0), min(i + 1, n - 1)
                    ux, uz = pos[hi][0] - pos[lo][0], pos[hi][1] - pos[lo][1]
                    uu = ux * ux + uz * uz or 1e-9
                    for side, rest in ((n, math.pi / 2), (2 * n, -math.pi / 2)):
                        f = side + i
                        vx, vz = pos[f][0] - pos[i][0], pos[f][1] - pos[i][1]
                        vv = vx * vx + vz * vz or 1e-9
                        angle = math.atan2(ux * vz - uz * vx, ux * vx + uz * vz)
                        twist = -square * (angle - rest)
                        gf = (-vz / vv * twist, vx / vv * twist)          # on the flank cell
                        gu = (uz / uu * twist, -ux / uu * twist)          # on the bone ahead
                        force[f][0] += gf[0]; force[f][1] += gf[1]
                        force[i][0] -= gf[0]; force[i][1] -= gf[1]
                        force[hi][0] += gu[0]; force[hi][1] += gu[1]
                        force[lo][0] -= gu[0]; force[lo][1] -= gu[1]
            for i in range(1, n - 1):
                a, b, c = pos[i - 1], pos[i], pos[i + 1]
                e1 = (b[0] - a[0], b[1] - a[1]); e2 = (c[0] - b[0], c[1] - b[1])
                l1 = math.hypot(*e1) or 1e-3; l2 = math.hypot(*e2) or 1e-3
                d1 = (e1[0] / l1, e1[1] / l1); d2 = (e2[0] / l2, e2[1] / l2)
                bend = math.atan2(d1[0] * d2[1] - d1[1] * d2[0], d1[0] * d2[0] + d1[1] * d2[1])   # NEW: knows left from right
                swing = max(swing, abs(bend))
                twist = max(-torque, min(torque, -(bend - target[i]) * stiff))
                effort = twist                         # the muscle's share, before spring and stop
                twist -= spring * bend                 # NEW: the backbone's own spring toward straight
                halt = -STOP_STIFF * (bend - math.copysign(STOP, bend)) if abs(bend) > STOP else 0.0
                if not fair:
                    twist += halt
                ga = (-d1[1] / l1, d1[0] / l1); gc = (-d2[1] / l2, d2[0] / l2)
                gb = (-ga[0] - gc[0], -ga[1] - gc[1])
                if fair:
                    # NEW: the three pushes of a joint are held back together, so they still sum
                    # to nothing. Held back one by one (the engine) they push the body along.
                    most = max(math.hypot(*ga), math.hypot(*gb), math.hypot(*gc)) * abs(twist)
                    if most > JOINT_PUSH:
                        twist *= JOINT_PUSH / most
                        effort *= JOINT_PUSH / most
                    twist += halt                      # bone on bone is not held back
                # Work the muscle does this instant: its twist times how fast the joint is turning.
                # A muscle gets nothing back when it is driven the other way.
                turning = sum(g[0] * vel[j][0] + g[1] * vel[j][1] for j, g in ((i - 1, ga), (i, gb), (i + 1, gc)))
                work += max(0.0, effort * turning) * SUB
                for j, g in ((i - 1, ga), (i, gb), (i + 1, gc)):
                    px, pz = g[0] * twist, g[1] * twist
                    size = math.hypot(px, pz)
                    if not fair and size > JOINT_PUSH and abs(bend) <= STOP:
                        px, pz = px * JOINT_PUSH / size, pz * JOINT_PUSH / size
                    force[j][0] += px; force[j][1] += pz
            # The water against each cell's movement within the body.
            dx = sum(m * w[0] for m, w in zip(mass, vel)) / total
            dz = sum(m * w[1] for m, w in zip(mass, vel)) / total
            if water == "engine":
                for i, w in enumerate(vel):
                    keep = 1 / (1 + single[i])
                    w[0] = dx + (w[0] - dx) * keep; w[1] = dz + (w[1] - dz) * keep
            for i, w in enumerate(vel):
                w[0] += force[i][0] * SUB / mass[i]; w[1] += force[i][1] * SUB / mass[i]
                # "engine": the body as a whole is held by its reach, and each cell is held, with
                # no push on the body, against moving within it. "cells" (NEW): each cell is held
                # by the water it meets, whether the body or the cell is what is moving.
                ratio = min(single[i], whole) if water == "engine" else single[i]
                sx, sz = -w[0], -w[1]                # the still water's speed past the cell
                if shaped:
                    hx, hz = sx, sz
                    for tx, tz in around[i]:
                        along = sx * tx + sz * tz
                        hx -= SHIELD * along * tx; hz -= SHIELD * along * tz
                    ax, az = axes[i]
                    across = -sx * az + sz * ax
                    hx += FIN_SPAN * fin[i] * across * -az; hz += FIN_SPAN * fin[i] * across * ax
                    share = ratio / (1 + ratio * (1 + FIN_SPAN * fin[i]))
                    w[0] += hx * share; w[1] += hz * share
                else:
                    w[0] += sx * ratio / (1 + ratio); w[1] += sz * ratio / (1 + ratio)
                pos[i][0] += w[0] * SUB; pos[i][1] += w[1] * SUB
        if trace and 60 <= tick < 66:
            bends = []
            for i in range(1, n - 1):
                a, b, c = pos[i - 1], pos[i], pos[i + 1]
                d1 = (b[0] - a[0], b[1] - a[1]); d2 = (c[0] - b[0], c[1] - b[1])
                bends.append(math.atan2(d1[0] * d2[1] - d1[1] * d2[0], d1[0] * d2[0] + d1[1] * d2[1]))
            print("      tick", tick, "bends", " ".join(f"{x:+.2f}" for x in bends), " aims", " ".join(f"{x:+.2f}" for x in target[1:n - 1]))
        row = []
        for i in range(1, n - 1):
            a, b, c = pos[i - 1], pos[i], pos[i + 1]
            d1 = (b[0] - a[0], b[1] - a[1]); d2 = (c[0] - b[0], c[1] - b[1])
            row.append(math.atan2(d1[0] * d2[1] - d1[1] * d2[0], d1[0] * d2[0] + d1[1] * d2[1]))
        seen_bends.append(row)
        tick += 1
        if sine and tick % sine[0] == 0:
            head_fires.append(tick)
            marks.append(centre())
            work_marks.append(work)
    if len(marks) < 12:
        return None
    first, last = marks[10], marks[-1]
    beats_run = len(marks) - 11
    period = (head_fires[-1] - head_fires[10]) / beats_run
    went = math.hypot(last[0] - first[0], last[1] - first[1]) / beats_run / length
    heading = (last[0] - first[0]) / (math.hypot(last[0] - first[0], last[1] - first[1]) or 1)
    # The wave the body itself makes: each joint's swing at the beat's own rate, and how much
    # later each swings than the one ahead. Waves on the body, counted head to tail.
    t0, t1 = head_fires[10], head_fires[-1]
    parts = []
    for j in range(n - 2):
        re = sum(seen_bends[t][j] * math.cos(2 * math.pi * t / period) for t in range(t0, t1))
        im = sum(seen_bends[t][j] * math.sin(2 * math.pi * t / period) for t in range(t0, t1))
        parts.append((math.hypot(re, im) * 2 / max(1, t1 - t0), math.atan2(im, re)))
    turns = 0.0
    for (_, p0), (_, p1) in zip(parts, parts[1:]):
        turns += ((p1 - p0 + math.pi) % (2 * math.pi) - math.pi) / (2 * math.pi)
    swimmer.wave = turns
    swimmer.swings = [round(a, 2) for a, _ in parts]
    swimmer.work = (work_marks[-1] - work_marks[10]) / beats_run / period * 4      # per cycle
    swimmer.speed = went * length / period * 4                                      # units a cycle
    return went, swing, stretch, period, heading


def part_c():
    global JOINT_PUSH
    print("\nC. The whole swimmer with the engine's numbers, nerves as in B2 pulling every joint from both")
    print("   sides. Old water (one drag every way) is the check on the model: nothing can travel in it,")
    print("   so anything but 0 there is a fault. All together = every nerve cell firing at once.")
    print("   travel = body lengths a beat; cycle = body lengths a cycle; stretch = the longest any tie")
    print("   got beside its rest length (past 2 it tears)\n")
    print("   C1. The body first planned: three cells wide, the engine's joint (twist capped at 1,")
    print("       each of a joint's three pushes held back singly, no spring of its own)")
    for label, kw in (
        ("muscle in the flank ties, fins on the flanks", dict(fins="all")),
        ("muscle at the joints, soft flanks, fins on the flanks", dict(fins="all", flank_pull=False, soft=3.0)),
    ):
        went, _, stretch, period, _ = swimmer(**kw)
        print(f"       {label:56s} travel {went:5.3f}  cycle {went / period * 4:5.3f}  stretch {stretch:4.2f}")
    print("\n   C2. A single line of nine bones with a fin on each. NEW in the joint: it knows left from")
    print("       right, has a spring of its own (2) toward straight, a hard stop at 0.9, an aim that")
    print("       moves half-way each tick, and its three pushes held back together.")
    print("       twist cap  stiffness   old water   no fins   fins   tail fins only   all together   stretch")
    fair = dict(flanks=False, fair=True, spring=2.0, water="cells", ease=0.5)
    for push, torque, stiff in ((20.0, 1.0, 15.0), (20.0, 4.4, 60.0), (60.0, 13.0, 15.0), (60.0, 13.0, 60.0), (60.0, 13.0, 240.0)):
        JOINT_PUSH = push
        kw = dict(torque=torque, stiff=stiff, **fair)
        row = [swimmer(shaped=False, fins="all", **kw), swimmer(**kw), swimmer(fins="all", **kw),
               swimmer(fins="tail", **kw), swimmer(fins="all", fall=0.0, hold=0.0, **kw)]
        print(f"       {torque:9g}  {stiff:9g}   {row[0][0]:9.3f}   {row[1][0]:7.3f}   {row[2][0]:4.3f}   {row[3][0]:14.3f}   {row[4][0]:12.3f}   {row[2][2]:7.2f}")
    print("\n   C3. The same at twist 13, stiffness 60, with other bodies")
    JOINT_PUSH = 60.0
    kw = dict(torque=13.0, stiff=60.0, fair=True, spring=2.0, water="cells", ease=0.5)
    for label, more in (
        ("single line of 9 bones, a fin on each", dict(flanks=False, fins="all")),
        ("  with a full-size cell on each flank", dict(flank_pull=False, soft=3.0, fins="spine")),
        ("  with a nerve cord on top, cells 0.7 wide", dict(flank_pull=False, soft=3.0, small=0.7, cord=True, rib=100.0, fins="spine")),
        ("    the cord's hold read once a tick (engine)", dict(flank_pull=False, soft=3.0, small=0.7, cord=True, rib=100.0, fins="spine", sampled=True, hold=0.2)),
        ("    the same in old water", dict(flank_pull=False, soft=3.0, small=0.7, cord=True, rib=100.0, fins="spine", sampled=True, hold=0.2, shaped=False)),
        ("5 bones", dict(flanks=False, fins="all", n=5)),
        ("13 bones, channels falling 5%", dict(flanks=False, fins="all", n=13, fall=0.05)),
        ("13 bones, channels falling 10%", dict(flanks=False, fins="all", n=13, fall=0.1)),
    ):
        got = swimmer(**kw, **more)
        if got is None:
            print(f"       {label:48s} no steady beat")
            continue
        went, _, stretch, period, heading = got
        print(f"       {label:48s} travel {went:5.3f}  cycle {went / period * 4:5.3f}  head first {-heading:+4.1f}  stretch {stretch:4.2f}")
    print("\n   C4. What the stroke costs. Single line of 9 finned bones, stiffness 60. Channels = how")
    print("       much channel the nerve cells have in all (less is a slower beat). Fuel is the joints'")
    print("       muscle work priced as the engine prices a muscle's (0.08 a unit of work).")
    print("       twist cap  channels   beat (ticks)   lengths a beat   lengths a cycle   fuel a cycle   fuel a body length")
    for push, torque in ((20.0, 1.0), (20.0, 4.4), (60.0, 13.0)):
        JOINT_PUSH = push
        for pace in (1.0, 0.5, 0.25, 0.125):
            got = swimmer(flanks=False, fins="all", fair=True, spring=2.0, water="cells", ease=0.5, stiff=60.0,
                          torque=torque, pace=pace, beats=14)
            if got is None:
                continue
            went, _, _, period, _ = got
            cycle = went / period * 4
            fuel = swimmer.work * 0.08
            print(f"       {torque:9g}  {pace:8g}   {period:12.1f}   {went:14.3f}   {cycle:15.3f}   {fuel:12.2f}   {fuel / max(cycle, 1e-9):18.1f}")
    JOINT_PUSH = 20.0
    print("\n   C5. The tadpole as a frog builds it: a springy rod, a muscle block on each flank of every")
    print("       segment pulling on the next, a fin fold on the rod, no muscle at the joints, slow beat.")
    print("       body wave = waves the body itself makes, head to tail; swing = each joint's, head first")
    tad = dict(fair=True, water="cells", beats=16, flank_pull=True, swing_to=0.0, stiff=0.0001, torque=1.0,
               pace=0.25, fins="spine", spring=2.0)
    for label, kw in (
        ("9 segments, ties that hold only a length", dict(n=9, fall=0.1)),
        ("9 segments, blocks held square to the rod", dict(n=9, fall=0.1, square=30.0)),
        ("  the same in old water", dict(n=9, fall=0.1, square=30.0, shaped=False)),
        ("  the same, every nerve cell firing together", dict(n=9, fall=0.0, hold=0.0, square=30.0)),
        ("18 segments, a quarter of a nerve wave on the body", dict(n=18, fall=0.03, square=30.0)),
        ("18 segments, nearly two nerve waves on the body", dict(n=18, fall=0.12, square=30.0)),
    ):
        got = swimmer(**tad, **kw)
        if got is None:
            print(f"       {label:50s} no steady beat")
            continue
        went, _, _, _, heading = got
        print(f"       {label:50s} travel {went:5.3f}  head first {-heading:+4.1f}  body wave {swimmer.wave:+5.2f}  swings {swimmer.swings}")


if __name__ == "__main__":
    part_a()
    part_b()
    part_c()
