"""Draws a recorded pond as a video and a few stills. Needs Pillow and ffmpeg.

Record first, from engine/ (about two minutes, 300 MB):
  SPECIES=fish FILM=/tmp/fish.jsonl cargo test --release --lib -- probe_film --ignored
Then draw, top view, the camera following the largest body:
  python3 scripts/pond_film.py /tmp/fish.jsonl films/fish "Fish in a pond"


usage: film.py recording.jsonl out_prefix "Title" [view_units [frames_a_second [last_frame]]]
"""
import json
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

W, H, SS = 1280, 720, 2
VIEW = float(sys.argv[4]) if len(sys.argv) > 4 else 18.0
S = W / VIEW  # pixels per world unit
SRC, OUT, TITLE = sys.argv[1], sys.argv[2], sys.argv[3]
FPS = sys.argv[5] if len(sys.argv) > 5 else "30"
LAST = int(sys.argv[6]) if len(sys.argv) > 6 else 10**9

MOUTH, BUSY, EYE, BONE, FIN, MUSCLE, FIRING, GAMETE, ZYGOTE, STEM = (1 << k for k in range(10))
WATER = (9, 26, 38)
FLESH, BONY, FINNED, MOUTHED = (216, 164, 143), (239, 230, 201), (111, 195, 217), (228, 87, 46)
PREY = {"Plankton": (63, 143, 90), "Cyanobacterium": (47, 143, 143)}
OTHER = (90, 122, 58)


def font(size):
    for path in ("/System/Library/Fonts/Helvetica.ttc", "/System/Library/Fonts/SFNS.ttf"):
        try:
            return ImageFont.truetype(path, size * SS)
        except OSError:
            pass
    return ImageFont.load_default()


BIG, MID, SMALL = font(26), font(17), font(13)

with open(SRC, "rb") as f:
    f.seek(0, 2)
    f.seek(max(0, f.tell() - 4096))
    names = json.loads(f.read().decode().strip().splitlines()[-1])["names"]


def find(parent, i):
    while parent[i] != i:
        parent[i] = parent[parent[i]]
        i = parent[i]
    return i


ffmpeg = subprocess.Popen(
    ["ffmpeg", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", FPS,
     "-i", "-", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "21", OUT + ".mp4"],
    stdin=subprocess.PIPE,
)

cam = None
anchor = None
before = {}  # id -> (meals, size of the body it was in)
flashes = {}  # id -> frames left
freed = {}  # id -> frames left
meals = births = released = shed = fertilised = 0
stills = {}
spot = None  # (id, frames left): a fertilised egg the camera goes to look at
bounds = None
shots = 0

for line in open(SRC):
    frame = json.loads(line)
    if "names" in frame or shots >= LAST:
        break
    cells, ties = frame["cells"], frame["ties"]
    mine = [k for k, c in enumerate(cells) if c[5] == 0]
    if bounds is None:
        xs, zs = [c[1] for c in cells], [c[2] for c in cells]
        bounds = (min(min(xs) - 1, -30), min(min(zs) - 1, -30), max(max(xs) + 1, 30), max(max(zs) + 1, 30))
    # Bodies: cells of this kind joined by ties.
    parent = {k: k for k in mine}
    for a, b, _ in ties:
        if a in parent and b in parent:
            parent[find(parent, a)] = find(parent, b)
    groups = {}
    for k in mine:
        groups.setdefault(find(parent, k), []).append(k)
    size = {k: len(g) for g in groups.values() for k in g}
    bodies = sorted((g for g in groups.values() if len(g) >= 2), key=lambda g: (-len(g), min(cells[k][0] for k in g)))
    # What happened since the last frame.
    now = {}
    for k in mine:
        c = cells[k]
        was = before.get(c[0])
        if was is None:
            if before and c[6] & GAMETE:
                shed += 1
            elif before and c[6] & ZYGOTE:
                fertilised += 1
                freed[c[0]] = 60
                spot = (c[0], 75)
            elif before:
                births += 1
                if size[k] == 1:
                    released += 1
                    freed[c[0]] = 45
        else:
            if c[7] > was[0]:
                meals += c[7] - was[0]
                flashes[c[0]] = 6
            if size[k] == 1 and was[1] >= 2 and not c[6] & (GAMETE | ZYGOTE):
                released += 1
                freed[c[0]] = 45
        now[c[0]] = (c[7], size[k])
    before = now
    # The camera stays with one body while it is among the largest.
    where = {cells[k][0]: k for k in mine}
    follow = None
    if anchor in where and bodies and size[where[anchor]] >= 0.6 * len(bodies[0]):
        follow = groups[find(parent, where[anchor])]
    elif bodies:
        follow = bodies[0]
        anchor = min(cells[k][0] for k in follow)
    elif anchor in where:
        follow = [where[anchor]]
    elif mine:
        anchor = min(where)
        follow = [where[anchor]]
    if spot and spot[1] > 0 and spot[0] in where:
        follow = [where[spot[0]]]
        spot = (spot[0], spot[1] - 1)
    if follow:
        tx = sum(cells[k][1] for k in follow) / len(follow)
        tz = sum(cells[k][2] for k in follow) / len(follow)
        cam = (tx, tz) if cam is None else (cam[0] + (tx - cam[0]) * 0.08, cam[1] + (tz - cam[1]) * 0.08)
    if cam is None:
        cam = (0.0, 0.0)

    img = Image.new("RGB", (W * SS, H * SS), WATER)
    d = ImageDraw.Draw(img, "RGBA")

    def at(x, z):
        return ((x - cam[0]) * S + W / 2) * SS, ((z - cam[1]) * S + H / 2 + 30) * SS

    def seen(x, z, r):
        return abs(x - cam[0]) < VIEW / 2 + r and abs(z - cam[1]) < VIEW * H / W / 2 + r

    def disc(x, z, r, fill, outline=None, width=1):
        px, pz = at(x, z)
        pr = r * S * SS
        d.ellipse((px - pr, pz - pr, px + pr, pz + pr), fill=fill, outline=outline, width=width * SS)

    # Ties first, under the cells.
    for a, b, act in ties:
        ca, cb = cells[a], cells[b]
        if not (seen(ca[1], ca[2], 1) or seen(cb[1], cb[2], 1)):
            continue
        if ca[5] == 0 and cb[5] == 0:
            continue
        width = max(2, int(min(ca[4], cb[4]) * S * 0.5)) * SS
        d.line((*at(ca[1], ca[2]), *at(cb[1], cb[2])), fill=(40, 70, 60), width=width)
    # Everything that is not the animal: its food and its neighbours.
    for c in cells:
        if c[5] != 0 and seen(c[1], c[2], c[4]):
            disc(c[1], c[2], c[4], PREY.get(names[c[5]], OTHER))
    # The animal. Which way each part of its middle line faces is read off
    # the part it is tied to, to draw a head's two eyes and a tail's fin.
    beside = {}
    for a, b, _ in ties:
        if a in parent and b in parent:
            beside.setdefault(a, []).append(b)
            beside.setdefault(b, []).append(a)

    def facing(k):
        c = cells[k]
        for n in beside.get(k, []):
            if cells[n][8] == 0:
                dx, dz = c[1] - cells[n][1], c[2] - cells[n][2]
                length = (dx * dx + dz * dz) ** 0.5
                if length > 1e-6:
                    return dx / length, dz / length
        return None

    for k in sorted(mine, key=lambda k: cells[k][3]):
        c = cells[k]
        if not seen(c[1], c[2], c[4] + 1):
            continue
        flags = c[6]
        way = facing(k) if c[8] == 0 else None
        if way and flags & FIN and flags & STEM and size[k] >= 3:
            # The tail fin: a fan behind the last part of the line.
            (fx, fz), r = way, c[4]
            tip = [at(c[1], c[2]), at(c[1] + fx * r * 2.2 - fz * r * 1.3, c[2] + fz * r * 2.2 + fx * r * 1.3),
                   at(c[1] + fx * r * 1.5, c[2] + fz * r * 1.5),
                   at(c[1] + fx * r * 2.2 + fz * r * 1.3, c[2] + fz * r * 2.2 - fx * r * 1.3)]
            d.polygon(tip, fill=FINNED + (150,), outline=(40, 80, 95))
        fill = BONY if flags & BONE else FINNED if flags & FIN else FLESH
        if flags & MOUTH:
            disc(c[1], c[2], c[4], fill + (175,), MOUTHED, 4)
        else:
            disc(c[1], c[2], c[4], fill + (175,), (60, 36, 30), 2)
        if flags & BUSY:
            disc(c[1], c[2], c[4] * 0.5, (255, 222, 89))
        if flags & EYE and way and flags & MOUTH and c[9] == 0:
            # A head: an eye to each side, the mouth in front.
            (fx, fz), r = way, c[4]
            for side in (-1, 1):
                disc(c[1] + fx * r * 0.2 - side * fz * r * 0.55, c[2] + fz * r * 0.2 + side * fx * r * 0.55, r * 0.2, (15, 15, 20))
            disc(c[1] + fx * r * 0.8, c[2] + fz * r * 0.8, r * 0.22, (120, 30, 20))
        elif flags & EYE and not flags & MOUTH:
            disc(c[1], c[2], c[4] * 0.28, (15, 15, 20))
        if flags & STEM and size[k] >= 2:
            disc(c[1], c[2], c[4] * 0.3, (120, 220, 120))
        if flags & GAMETE:
            disc(c[1], c[2], c[4] + 0.12, None, (200, 120, 255), 3)
        elif flags & ZYGOTE:
            disc(c[1], c[2], c[4] + 0.12, None, (255, 105, 180), 4)
        elif size[k] == 1:
            disc(c[1], c[2], c[4] + 0.12, None, (255, 214, 64), 2)
    for a, b, act in ties:
        ca, cb = cells[a], cells[b]
        if ca[5] == 0 and cb[5] == 0 and (seen(ca[1], ca[2], 1) or seen(cb[1], cb[2], 1)):
            colour = (255, 60, 50) if act > 0.05 else (40, 24, 20)
            d.line((*at(ca[1], ca[2]), *at(cb[1], cb[2])), fill=colour, width=(5 if act > 0.05 else 3) * SS)
    for ident, left in list(freed.items()):
        if ident in where and size[where[ident]] == 1:
            c = cells[where[ident]]
            px, pz = at(c[1], c[2])
            egg = c[6] & ZYGOTE
            d.text((px + (c[4] + 0.2) * S * SS, pz - 9 * SS), "fertilised egg" if egg else "young",
                   font=SMALL, fill=(255, 105, 180) if egg else (255, 214, 64))
        freed[ident] = left - 1
        if left <= 1:
            del freed[ident]
    for ident, left in list(flashes.items()):
        if ident in where:
            c = cells[where[ident]]
            disc(c[1], c[2], c[4] + 0.06 + (6 - left) * 0.03, None, (255, 255, 255, 40 * left), 2)
        flashes[ident] = left - 1
        if left <= 1:
            del flashes[ident]

    # A map of the whole pond, the animal's cells bright on it.
    m, pad = 150 * SS, 16 * SS
    mx, mz = W * SS - m - pad, H * SS - m - pad
    d.rectangle((mx, mz, mx + m, mz + m), fill=(6, 18, 27), outline=(70, 100, 110), width=SS)
    span = max(bounds[2] - bounds[0], bounds[3] - bounds[1])

    def small(x, z):
        return mx + (x - bounds[0]) / span * m, mz + (z - bounds[1]) / span * m

    for c in cells[::3]:
        if c[5] != 0:
            px, pz = small(c[1], c[2])
            d.point((px, pz), fill=(40, 90, 70))
    for k in mine:
        px, pz = small(cells[k][1], cells[k][2])
        d.ellipse((px - 2 * SS, pz - 2 * SS, px + 2 * SS, pz + 2 * SS), fill=(255, 190, 160))
    a, b = small(cam[0] - VIEW / 2, cam[1] - VIEW * H / W / 2), small(cam[0] + VIEW / 2, cam[1] + VIEW * H / W / 2)
    d.rectangle((*a, *b), outline=(255, 255, 255), width=SS)

    d.rectangle((0, 0, W * SS, 102 * SS), fill=(6, 18, 27, 200))
    d.rectangle((0, (H - 42) * SS, (W - 190) * SS, H * SS), fill=(6, 18, 27, 200))
    d.text((20 * SS, 14 * SS), TITLE, font=BIG, fill=(240, 240, 240))
    loose = sum(1 for g in groups.values() if len(g) == 1)
    d.text(
        (20 * SS, 50 * SS),
        f"cycle {frame['t'] // 4}   cells {len(mine)}   bodies {len(bodies)}"
        f"   largest {len(bodies[0]) if bodies else 1}   loose cells {loose}",
        font=MID, fill=(200, 215, 220),
    )
    d.text(
        (20 * SS, 74 * SS),
        f"meals {meals}   cells born {births}   gametes shed {shed}   eggs fertilised {fertilised}"
        f"   cells come loose {released}",
        font=MID, fill=(200, 215, 220),
    )
    legend = [(MOUTHED, "mouth"), ((255, 222, 89), "digesting"), ((15, 15, 20), "eye"), ((120, 220, 120), "growing tip"), (BONY, "bone"), (FINNED, "fin or foot"), (FLESH, "flesh"),
              ((255, 90, 77), "muscle pulling (line)"), ((200, 120, 255), "gamete"), ((255, 105, 180), "fertilised egg"),
              ((255, 214, 64), "loose cell"), (PREY["Plankton"], "plankton"),
              (PREY["Cyanobacterium"], "cyanobacteria"), (OTHER, "mat")]
    x = 20 * SS
    for colour, label in legend:
        d.ellipse((x, (H - 30) * SS, x + 12 * SS, (H - 18) * SS), fill=colour)
        d.text((x + 17 * SS, (H - 32) * SS), label, font=SMALL, fill=(200, 215, 220))
        x += (17 + 9) * SS + int(d.textlength(label, font=SMALL))

    img = img.resize((W, H), Image.LANCZOS)
    ffmpeg.stdin.write(img.tobytes())
    shots += 1
    # Stills: a meal in view, a young cell just set free in view, and the end.
    in_view = lambda ident: ident in where and seen(cells[where[ident]][1], cells[where[ident]][2], 0)
    if "meal" not in stills and bodies and len(bodies[0]) >= 4 and any(v == 4 and in_view(i) for i, v in flashes.items()):
        stills["meal"] = img
    if "egg" not in stills and any(
            v == 50 and in_view(i) and cells[where[i]][6] & ZYGOTE for i, v in freed.items() if i in where):
        stills["egg"] = img
    stills["end"] = img

ffmpeg.stdin.close()
ffmpeg.wait()
for name, img in stills.items():
    img.save(f"{OUT}-{name}.png")
print(f"{TITLE}: {shots} frames; meals {meals}, cells born {births}, gametes shed {shed}, eggs fertilised {fertilised}, "
      f"cells come loose {released}; stills {sorted(stills)}")
