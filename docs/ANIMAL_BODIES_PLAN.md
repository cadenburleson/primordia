# Animal bodies that travel, eat and breed: research and plan

Written 2026-10-13 from web research and a paper model, before any engine change. The first part of package 1 has since been built (see "Built so far"); the rest is still a plan. It was made from web research and from reading the engine's source, after a night of trial changes that each failed and were taken out (recorded in `docs/GEOLOGIC_TIMELINE.md`, "One planet four times the size, counted honestly"). It replaces the build order given there.

Three kinds of statement are kept apart below: **sourced** (a paper read, or a search summary of one, listed at the end), **worked out here** (arithmetic from a sourced law), and **read in the engine** (with file and line).

## Built so far

2026-10-13, package 1's engine side, built at low priority. **Not rebuilt into the app:** `src/wasm` still holds the engine from before, so the app is unchanged. Suite: 186 pass, 0 fail, 42 set aside (five of them newly, each with its reason written on it).

In the engine now:

- **Drag by shape** (law 1). A cell in a body is covered along each tie and meets the water across it; a fin is a blade across the cell's own line. Taken over all directions a body is held as hard as it was. A line of nine holds the water 0.556 as hard end-on as broadside (the paper model said 0.556).
- **The one-way fin grip in water is gone** (law 2). The foot's grip on the ground and the wing's on the air are still there.
- **A backbone's joint** (law 5): between three cells the body's plan set in a line (or three hand-laid straight and level) it knows left from right, has a spring of its own and a stop both sideways and up and down, an aim that takes time, and a twist that goes with the size of the cells it joins.
- It is pulled from its two sides by nerve cells riding on it; or, with a nerve of its own and none riding, one way while that nerve fires and the other while it recovers. The second stands for a segment's two sides in one cell, as the joint motor already stands for two muscles.
- **A joint's three pushes are held back together**, for every joint, so they always sum to nothing.
- **Bone joined to bone along one line of the body does not come apart by chance,** and muscle between two bones turns the joint and does not shorten the bone.
- **Stiffness is measured against living tissue** (law 4, part), not against the water, fuel and fat a cell carries.
- **A body does not glue to itself** (law 6): cells with a place in a plan tie on contact, and are shared out at division, only where they were born next to each other.

**What swims in the engine, hand-built, 75 cycles each:**

| Body | Travel |
| --- | --- |
| Nine finned bones, each with its own nerve, a little slower toward the tail | **5.5 body lengths, head first** |
| The same with every nerve alike | 0.09, the other way |
| Nine finned bones with a nerve cell riding on each side of each (the tadpole's wiring) | 0.1 to 0.7, head first |

The paired wiring swims, but how far swings with the strength of the hold across and with chance, so its test is set aside as a probe; the single line is the swimmer the suite holds to. The first is 0.073 body lengths a cycle, four to twelve times what the old grip gave (0.006 to 0.018). It is not yet a result for the sim: the body is laid by hand, its nerve level is set by hand, and its fuel is refilled every tick. Its joints did about 3 of fuel a cycle of work, several times what a body that size takes in, so as it stands it is a sprint.

- **How fast a nerve beats is inherited now.** It goes by how much channel a cell carries for its size, up to a full set (`CHANNEL_SET`). A gene can ask for a quarter of a set and get a beat of about twenty ticks, which a body can follow; the starters, which carry a full set, beat as before.

**A Lancelet grown from an egg** (a 52nd starter, in the list and marked unfinished) is the first grown animal to move this way: a single line with one tie between neighbours, bone at its limit, a backbone joint at every place, and its head at the end it began at. In the nursery it went 0.063 units a cycle, 0.018 of its length, with a third of its cells' channel still to build.

**Not working yet:**

- **In the nursery the Lancelet is four cells long where it should be nine.** With its head at the old end it has the right fall in nerve channel for free: its oldest cells have had longest to build it (0.091 of tissue at the head, 0.015 at the growing tail). It was first put down to food not reaching the tail tip. That was a guess and it was wrong: the tip is the best-fed cell in the body (tissue 0.57, fuel 0.77, against 0.34 to 0.38 and about 0.5 in the cells behind it), and the body the keeper takes is only 70 cycles old. Why the nursery ends with a young one is not known.
- **In a pond it lives and breeds.** Over 400 cycles it went from 36 cells to 103 in bodies of up to 20, and took in 2.4 to 3.9 times what it burned: the first animal here to clear the "twice what it burns" mark. But pond water has almost no mineral, so it lays no bone there and has no joints.

**In a pond with a sea's mineral in the water** (`SEA=0.09` on `probe_cast_lancelet`; the mineral is booked as weathered rock), over 400 cycles, paying for its own strokes, with a mouth at its head only:

| | |
| --- | --- |
| Cells | 8, 17, 18, 33 at each hundred cycles |
| Bodies | up to 13 cells |
| Joints | 35, of which 7 a backbone's; 28 being pulled by muscle at the count; bent up to 0.34 radian |
| Eaten against burned, a cell each cycle | 0.016 to 0.019 against 0.006 to 0.008: 2.4 to 3.1 times |
| Fastest body | 0.03 to 0.08 units a cycle |
| The books | all four balance at the end, energy among them |

This is the first animal grown by the sim that moves by muscle at bony joints, eats more than it spends and breeds. It is one kind in a pond, not a planet.

**A mouth on the tail, found and removed.** Caden asked why the mouth was on the tail. The head had the mouth and the eye, but the tail tip had a mouth as well: the gene that gives a lone cell a larva's mouth also switched on in any cell with a single neighbour, and the tail tip has one. An earlier count of this pond (117 cells, 4.7 times its burn, 65 mouths in its bodies) was that animal, eating with both ends and most of its length; the table above is with the stray mouth gone. Released from the nursery, bodies raised for 300 and for 700 cycles then moved 1.35 and 0.65 units toward the mouth end in 40 cycles, so it goes head first; before the fix one went 1.42 units tail first.

**Why its bodies were not clean lines, and the rule that fixed it.** Printing how its pond bodies were tied showed lines out of order: a head tied to the fourth cell, the second tied to the fifth. A body tumbles as it grows, a cell's heading is kept only as a direction on the map, and a tip whose heading had come to face inward set its daughter down inside the line. Now the growing end of a planned body sets its daughter down on the side away from its one neighbour. After that every body in the pond was a line tied 0-1, 1-2, 2-3 with a backbone's joint at every place (5 of 5 joints, where it had been 7 of 35). The same rule lengthened the old starters' middle lines (a raised Sea scorpion is 40 cells with 18 on its middle line, where it was about 21 with 9).

**Why its bodies stopped at two to four cells, and the rule that fixed it.** The growing tip kept coming away, and the mother had by then taken up her place as a segment and could not grow another. Measured in the pond: every tie was already one piece of bone, so ties were not letting go. The loss was at birth. A newborn stayed tied with a chance equal to its glue times its mother's, and a cell at full glue reads 0.85 to 0.96, so one newborn in five was born loose however much glue the two held; eight in a row stay about one time in six. Real embryos do not work that way: a cell that divides inside a body stays in it, and what leaves is what stops making glue, or a gamete. So now, in a body with a plan (one whose cells have taken up places), a newborn stays without fail when its glue times its mother's is over 0.25 (`HELD`, `physical.rs`); below that, and in bodies with no plan, the old chance stands, since that chance is what sets a colony's size. Same pond, 400 cycles: 34 cells where there were 18, bodies of 8, 5, 4, 4, 2 where there were 4, 2, 2, 2, 2, and eight bodies in ten still have their growing tip where one in six did. This rule reaches every planned animal, not only the Lancelet; the suite has not been re-run since.

**How a real lancelet breeds** (looked up 2026-10-13). Its germ cells are set aside in the early embryo (maternal Vasa and Nanos in three species; one species suggests signalling instead, so not settled), and the same markers show in the tail bud, the growing end. Adults carry gonads in the segments of the middle of the body. They spawn in summer about half an hour after sunset, a group at a time within about ninety minutes, a male starting each night; a female sheds all her eggs at once, 1,400 to 12,800 of them; fertilisation is in open water. For the engine: gametes come from grown middle segments, never from the growing tip (a tip that made a gamete would count as having set a daughter down and stop growing); dusk is the cue to shed in step.

**Eggs and sperm, built 2026-10-13 for every planned animal** (Caden, 09:21: "why is there no eggs, sperm, or mating? ... all the videos you sent the organisms are just disintegrating. We are supposed to be creating life not killing it. Once something is born, its main function is to grow and reproduce"). Until then every animal bred by a cell falling off its body, which is the same thing as coming apart.
- *Gametes.* A segment of the middle line that has put fat by sheds half of itself into the water, fat and all, as a gamete (`germ()` in `bestiary.rs`: a division gene and the gamete gene, both switched on by the cell's fat share, plus the fusing gene and a fat store). Two gametes from different bodies that touch fuse into an egg with both sets of genes; the egg feeds as a larva until it is large enough to cleave. A body still growing stores little fat, so it grows first and breeds when grown. Fat as the cue is how real animals time breeding (stores decide when gonads ripen). Without the cue the Lancelet shed gametes at three cells and reached 14 cells in 400 cycles; with it, 27.
- *Only a gene at work now makes a gamete* (`divide`): a fertilised egg carries what both gametes held and must not shed its first daughter as another.
- *Bodies hold together.* In a planned body whose young are gametes, a newborn stays (above) and a tie between two cells that both hold glue in earnest lasts until it is torn; it no longer lets go by chance (`held`, `physical.rs`). Before, a tie at full glue still let go about once in 400 cycles, so a twenty-tie animal broke somewhere every 20 to 25 cycles; an animal with no bone (the Worm) had nothing else holding it and was dead by cycle 50.
- *The nursery's keeper takes out shed gametes* (`raise_in`): otherwise young grow beside the animal being raised and cells of different young trade genes (an old rule: a cell with gene 8 takes up a gene from a touching cell of another individual), so the animal handed over was not as designed.
- Measured, one pond each, 400 cycles: Fish 37 cells in bodies of 8, 7, 6, 2, 2, 18 gametes, 3 eggs fertilised (before: 25 cells, none). Sea scorpion 48 cells, 19 gametes, 5 fertilised. Worm 19 cells in bodies of 5, 5, 4 and alive (before: dead by cycle 50), 4 gametes, none fertilised because it was set down as one body and a gamete needs one from another body. Lancelet 27 cells, 16 gametes, 3 fertilised, and a second-generation body of five grown from an egg. In the nursery every planned animal sheds gametes (27 to 91 in 240 cycles). Suite 186 pass, 0 fail, 45 ignored.
- Still wrong: gametes are all one size (no small sperm and large egg); nothing brings two gametes together but chance and crowding near food (no scent of an egg, no shedding in step at dusk, no mating by contact for the arthropods); an unfertilised gamete lives on as a feeding cell instead of dying; pieces at limb ends still come away by design (their glue is switched off there), which is the old way of breeding and should go once limbs have a set length; the animals are still clumps, not fish or scorpion shapes.

**Shapes, built 2026-10-13 for the Fish, Sea scorpion and Worm** (Caden, 09:55: "we deff need them to have shapes of the organisms"). Each is now built on the Lancelet's body, the one that grows as a clean straight jointed line with its head at the end it began at (`lined` and `shaped` in `bestiary.rs`), and given its outline the way an embryo gives a body one: regions, part sizes, and limbs budded from set places.
- *Fish:* a head with eyes and mouth, a pair of fins set by the head (one part to each side), a trunk that tapers to the tail, a tail fin. Twelve parts.
- *Sea scorpion:* a head, three front segments each bearing a pair of two-part limbs (the last pair ending in paddles), nine armoured segments tapering to the tail. About 26 parts.
- *Worm:* a line of sixteen soft segments all of a size, no bone, limbs or fins.
- *How:* a second fate marks the front region (as Hox genes mark regions); a part's size is set by its place (`SIZE` read from `AXIAL` once a part is `PASSED`); a limb is a sideways daughter of a front part, and neither lengthens into a second body nor keeps budding (`OUT` on the division genes); organs of the head are on the middle line only.
- *The mouth is the front third* (`front` in `shaped`), with no muscle in it: each front part strains the water for itself, as a lancelet's gill basket does, and the body swims with the rest of its length. Found the hard way: the engine opens joined mouth parts that have muscle as one jaw, and a jaw that size lets plankton out with the water (`LEAK`), so a head-only or a muscled mouth region caught nothing and the animal starved.
- *Engine laws this needed:* a limb bud is tied to the part it grew from and to nothing else (sharing ties by nearness handed the body's next segment to the bud); cells of a planned body tie on contact only to plan neighbours along the middle line or along one limb, and only within one body (twins from one egg had been growing back together at the head); a tie in a planned body is as strong as the area it joins, as a joint's push already was (big parts were being torn off by the body's own strokes); a muscle may spend only its cell's energy above a quarter of what it can hold (`spare`; one hard stroke could spend a cell to death, which cut bodies in two); a fully driven gamete gene makes nothing but gametes and waits until it can.
- *The Fish's gametes* are cued by being well fed (energy input over 0.7), not by fat: every trunk part of it is swimming muscle and burns fat as fast as it stores it.
- *Measured.* Nursery, one egg, fed: the Fish is one body of 12 with its fins; it sheds its first gamete at cycle 83 and by cycle 400 there are 345 cells in 49 bodies, 184 gametes shed and 70 eggs fertilised. Sea scorpion: one body of 26, first gamete at cycle 354. Worm: one line of 16 (or twins). Ponds, 400 cycles, feeding themselves: Fish 72 cells at the start and 83 at cycle 300, 51 gametes, 17 eggs fertilised, some adults down from 12 parts to 7; Sea scorpion 80 to 141 (186 at cycle 400), 6 gametes, none fertilised, bodies down from 20 parts to 16, 9, 8; Worm five bodies of 11 that stay whole throughout, 5 gametes, none fertilised, no growth. Suite 186 pass, 0 fail, 45 ignored.
- *Wrong or unfinished:* one body's own gametes can fuse with each other (a gamete inherits its mother's "was tied" mark and is given a new origin of its own on its first tick; the rule meant to prevent this is in `fertilize`), and there are no sexes; fed in a pond a shaped Fish takes in about four fifths of what it burns, so adults lose parts; the Sea scorpion still drops pieces; the Worm has no skeleton to bend against (no bone, and nothing stands in for a worm's fluid skeleton) so it barely moves; the mouth-as-front-third is right for a filter feeder and wrong for a hunter, which needs a jaw and prey big enough to be worth it; the Trilobite and the rest of the planned cast are still built the old way.

Open: with newborns now staying, the Lancelet has no way to breed until its segments make gametes; its largest body (8) had cells tied out of order again; released bodies lose a third to a half of their cells in 40 cycles; and it swims slowly (0.003 to 0.007 of its length a cycle when released).

**The old starters under the new water** (plain pond, 400 cycles): Fish 76 cells, eating 1.3 times its burn; Worm 38 cells, 2 times; Sea scorpion 49 cells, 1.3 times; Trilobite 245 cells but eating only 0.6 of what it burns. So three of the four still feed themselves; "cannot feed" was too strong for all but the Trilobite.

- **The old cast** still squeezes where it should bend. Rebuilding those bodies in the frog's order is the rest of package 1.
- **Raised bodies came out smaller** after the first build (sea scorpions of 21 cells where they were about 32); why was not found.

## The finding in short

In this engine no way of moving a body can make it travel except a one-way grip, and the one-way grip is not physics.

- Water resists every cell of a body equally in every direction (`physical.rs:3464`: one drag figure for the whole body, shared out by mass). With that, the water's total push on a body depends only on how fast the body's centre is moving. No bending, squeezing, waving or rowing can produce thrust. (Worked out here; it is the standard result that thrust from a wave is proportional to the difference between sideways and lengthwise drag, and the difference is zero.)
- All travel measured so far came from `fin_drag` (`physical.rs:3231`), which resists a finned cell slipping backward and not forward, and from the same rule for a foot on the ground (`physical.rs:4003`) and a wing in air (`physical.rs:3885`). No real fin, foot or wing is a valve. It is why a squeeze "kept a twentieth" of its stroke and why nothing passed 0.02 body lengths a cycle.
- Real animals travel because of three things at once: **the medium pushes back unequally by direction**, **a skeleton that muscle cannot shorten turns pulling into bending**, and **muscles fire out of step, as a wave**. Last night's trials each changed one of the three (a stretch reflex, a stiffness change, a new body) and each failed. They could not have worked singly.

So the three are built together, with the bodies that use them, before anything is run. A paper model (below) says what to expect. With the engine's own numbers, the body first planned goes 0.001 body lengths a beat; a single line of nine finned bones with a mended joint goes 0.17, which is 0.14 a cycle against 0.006 to 0.018 today. A beat is about 1.2 cycles, so that is 0.08 and 0.13 a cycle, against 0.006 to 0.018 today.

## What the research says, and what the engine does

### Water

| | |
| --- | --- |
| Real (sourced) | A slender body meets more drag moving sideways than lengthwise: twice as much for a thin filament (Gray and Hancock 1955), about 1.4 to 1.5 for the nematode. A wave passed down such a body pushes it the other way. A motion that looks the same run backward (open and shut) gives no net travel in thick water: the "scallop theorem". Small swimmers (krill, copepods) row with many paddles in a wave, spread on the power stroke and folded on the return. Fish at larger sizes also use the water's inertia and reach a tail-beat ratio (Strouhal number) of 0.2 to 0.4, tail swing about a fifth of body length. |
| Other simulators (read) | Karl Sims (1994): "for each exposed moving surface, a viscous force resists the normal component of its velocity, proportional to its surface area and normal velocity magnitude." That one rule gave paddlers, tail-waggers, scullers with flippers and many-jointed water-snakes. He warns that any energy leak in the physics "will inevitably be discovered and exploited". |
| Worked out here | For a wave of height *a* and wavenumber *k* on a body whose sideways drag is ρ times its lengthwise drag, speed over wave speed is β(ρ−1)/(1+ρβ) with β = a²k²/2. One wave on the body with a swing of 0.16 body lengths: 0.25 body lengths a beat at ρ = 2, 0.12 at ρ = 1.4, 0.57 if fins raise ρ to 5. At ρ = 1 it is zero. |
| Engine (read) | ρ = 1 for every body. Drag is Stokes drag (proportional to speed), so the engine's sea is the thick-water case at every size. Cilia thrust is a separate force and is unaffected. |

### Skeleton and muscle

| | |
| --- | --- |
| Real (sourced) | Muscle only pulls, so muscles work in opposed pairs across a joint, on levers that trade force for speed. A fish's flank muscle bends the body because the notochord or backbone will not shorten: it is the body's stiffener, and a column fused too stiff slows the wave. Worms without bone use fluid held under pressure the same way. Joints stop: ligaments and the joint capsule go taut at the end of the range. A fish skeleton is under 2 to 3.5% of body mass; on land the share is larger and grows with size (mass to about the 1.1 power); an arthropod's shell keeps about the same share at any size. Sims gave every joint a limit with a restoring spring and every muscle a strength proportional to cross-section. The Cornell soft robots (Cheney and others 2013) use four materials only: muscle that changes volume 20% on a beat, a second muscle on the counter-beat, soft passive tissue and stiff passive bone, with no nerves at all; what travels is decided by where the patches are, and bodies with patches scattered at random do not travel. |
| Engine (read) | Muscle shortens a tie 22%, in line with Cornell's 20%. But a working muscle tie pulls with a stiffness of 300 where a tie between two bony cells has about 8 (`spring_k`, `physical.rs:4109`; bone adds 60 × mineral share, and the share is capped at 15% of tissue, `SKELETON_SHARE`). Muscle is forty times stronger than bone, the reverse of an animal, so every body shortens or folds. A joint's hold is capped at a torque of 1 and a push of 20 (`JOINT_TORQUE`, `JOINT_PUSH`), which muscle overwhelms: the 2.5 to 3 radian folds measured. Joints are made between any three bonded stiff cells in order of id, up to three a cell (`physical.rs:3276`), not along a limb. Bone is in every cell at the same low share instead of being its own part. Cells of one body glue to each other wherever they touch (`physical.rs:2644`), which is what tied the folded Lancelet shut. Stiffness is measured against mass including carried water (`rigidity`, `physical.rs:4773`); the `water * 3` in `mass` (`physical.rs:620`) is the water's density (`WATER_DENSITY` is 3), not an error. |

### Timing

| | |
| --- | --- |
| Real (sourced) | In the lamprey each segment fires about 1% of a beat after the one ahead, at any swimming speed, so with a hundred segments there is always one wave on the body. Left and right alternate by nerve cells that cross the cord and hold the other side down. In the nematode each stretch of body bends when it senses the stretch ahead bend. The muscle signal from one fish segment overlaps more than twenty neighbours: the unit is the wave, not the segment. Insects walk with a wave of leg movements passing from back to front at low speed and alternating tripods at high speed. |
| Engine (read) | Every nerve cell is driven directly by signal A in the water around it (`physiology.rs:576`), so every one is its own pacemaker, and the ties between them pull them into step. The whole body fires on one tick. Ties across the middle excite; nothing inhibits. Nerve speed crosses a body within one tick (`NERVE_PACE` 24). |

### Ground and air

| | |
| --- | --- |
| Real (sourced) | The Voxelyze simulator behind the Cornell work uses ordinary friction: a part stays put while the sideways push on it is under the friction coefficient times the load it bears, and slides against that coefficient times load once it moves, the same in every direction. Walking works because a leg in swing is lifted and bears no load. A wing loading near 10 N per square metre marks the edge of flight in stick insects; control of falling came before wings. |
| Engine (read) | Ground friction is a slowing in proportion to weight (`physical.rs:3996`), plus the one-way foot. A wing's lift is real (area × air density × speed squared); its forward drive is the one-way grip. |

### Feeding, young, water

| | |
| --- | --- |
| Real (sourced) | Guts are reactors: food is swallowed at once and digested over a holding time. In 42 worm species the gut was about 15% of body volume in hunters and 31% in mud-eaters; intake grows as body size to the 0.7 power. Hunters absorb 60 to 90% of what they eat, plant-eaters 10 to 80%. Predators are about 10 times their prey's mass among invertebrates and 100 times among cold-blooded vertebrates. An egg's mass grows as about the square root of the adult's in cold-blooded animals, and a clutch as the 0.75 to 1 power; many small eggs or few large is the trade. Yolky eggs carry the young to a complete small animal that starts to feed as the yolk runs out, and that first feeding is when most die. On land water leaves through the skin; a waxy cuticle is what stops it, and small bodies dry fastest. |
| Engine (measured earlier) | Mouths stand idle 75 to 100% of the time because bodies do not reach food: a gut would not help until they travel. A grown body takes in about what it burns, so it has nothing to fill eggs with. Young leave with 0.25 of tissue and need 0.58 to 1.1. Weight already ignores carried water under water and counts it on land. |

## Checked on paper before any engine change

`scripts/gait_model.py` is plain arithmetic, apart from the engine: about seven seconds on one core (`python3 -I scripts/gait_model.py`). Part A is a chain of stiff links in thick water with the joint angles given and the water deciding where it goes. Part B is the engine's own nerve equations, copied from `physiology.rs`, on two lines of nerve cells. It was written to settle on paper the questions that last night were put to the engine one build at a time.

The model checks itself: with the same drag every way a waving body travels 0.0000 body lengths a beat; with every joint swung at the same moment it travels 0.0000 at any drag; and a long body with three small waves goes 0.0204 where the smooth-wave formula says 0.0212.

What it settled:

| Question | Answer |
| --- | --- |
| How far does a nine-part body go per beat? | At the joint motor's present swing (0.45 radian each way) and sideways drag twice lengthwise: **0.10 body lengths a beat; 0.16 with a fin on the last two parts.** At a swing of 0.7: 0.16 and 0.25. A free body one wave long loses about a third to its own swaying against the smooth-wave formula. |
| Muscle blocks on the flanks, or the joint motor? | Flank blocks one part-width off the middle can bend a joint only about 0.22 radian with a 22% pull: 0.03 a beat for nine parts, and eighteen parts are needed to reach 0.10. **Bending is driven at the joints of the middle line**, pulled from the two sides by the nerve cells on the flanks. Where the muscle itself sits is open: in the flank ties it kinks the line (part C). |
| Does the nematode's drag ratio (1.4) do? | Under half the travel of a ratio of 2 (0.04 against 0.10). Fins matter. |
| How exact must the timing be? | Not very. Anywhere from a quarter of a wave to one whole wave on the body gives 0.08 to 0.12; half to three-quarters is best; a wave and a half gives 0.05; none gives exactly 0. |
| Can a body bend too far? | Yes. Eighteen parts at 0.7 radian curl over themselves and go 0.08, less than half of what they do at 0.45. |
| Fewest parts? | Three links on two hinges a quarter-beat apart go 0.04 a beat: enough for a limb, slow for a body. |
| **Can the engine's nerves carry a beat from one pacemaker down a line?** | **No.** With only the head cell driven, the next cell fires once or never, at every tie strength up to 1.2 (four times the strongest unsheathed tie). The first wording of law 7 would have failed in the engine. |
| What does give a wave with these nerves? | Leave every cell driven as it is now, make each a little slower than the one ahead (5 to 10% fewer channels from head to tail), and let each cell hold down the one across from it. Then every cell keeps one beat (4.9 ticks), left and right fire exactly half a beat apart, and 0.4 to 0.9 of a wave lies on the body, head to tail, for ties of 0.05 to 0.10, holds of 0.05 to 0.20 and drives of 0.25 to 0.35. A 20% fall is too much: the tail drops out of step. Thirteen parts want the 5% fall. |
| What breaks it? | Driving the head harder than the tail, with a strong hold across (0.15 or more), runs the wave tail to head: the animal would swim backward. A hold between the head cells only does not keep the sides in step. Both are avoided by the wiring above. |

### The whole swimmer on paper

Part C of the model puts the pieces together with the engine's own numbers (masses, tie springs and damping, the joint's twist and its caps, the water's hold) and the nerves of part B. Two checks run beside every case and both read 0.000 throughout: old water (one drag every way, in which nothing can travel) and every nerve cell firing at once (no wave).

**The body first planned does not travel**, even with the new water: 0.001 to 0.002 body lengths a beat, whether the muscle is in the flank ties or at the joints. Its joints do not follow the wave the nerves call for; the line folds into a zigzag, each joint bent the other way from the next. Three faults in the engine's joint, each found here:

- **Its twist is capped at 1** (`JOINT_TORQUE`). Once every joint is at the cap, the pushes on the cells between them cancel and nothing resists the line folding. This is the folding measured in the engine last night (2.5 to 3 radians), now with a cause. A real backbone has a spring of its own that muscle works against.
- **It cannot tell left from right.** Its angle is the plain angle between the two arms (`articulation.rs:57`), which is why the test chains are laid in a zigzag. A backbone needs a bend that is signed and smooth through straight.
- **It leaks momentum.** Each of a joint's three pushes is held back singly (`JOINT_PUSH`, `physical.rs:3797`), so when one is cut and the others are not they no longer sum to nothing and the body is pushed along by nothing. With the twist capped at 1 this bites only when two cells are pressed nearly into one; with a twist strong enough to swim it bites at every stroke, and on paper it made a body travel in old water. Held back together, the leak is gone.

**A body that does travel:** a single line of nine bones with a fin on each, the joint mended as in law 5, pulled from both sides by the nerves of part B.

| Joint twist cap | Joint stiffness | No fins | A fin on every bone | Fins on the last two only |
| --- | --- | --- | --- | --- |
| 1 (today) | 15 (today, a bone half mineral) | 0.003 | 0.011 | 0.013 |
| 4.4 | 60 | 0.013 | 0.079 | 0.063 |
| 13 | 15 | 0.005 | 0.066 | 0.051 |
| **13** | **60** | 0.027 | **0.173** | 0.132 |
| 13 | 240 | 0.025 | 0.173 | 0.120 |

(Body lengths a beat; a beat is 4.8 ticks, so 0.173 a beat is 0.14 a cycle, against 0.006 to 0.018 today.)

- **The twist a joint can give has to match the engine's own muscle.** A working muscle tie pulls with about 29; one cell-width from the joint that is a twist of 13. At 1 the joint is a thirteenth as strong as the muscle beside it.
- **The joint has to be stiff enough to follow a beat:** 60, four times what a half-mineral bone gives today. Past that nothing is gained.
- **Fins do most of it:** six times the travel of the same body without them. Fins on the tail alone give three-quarters of the full set.
- **Passengers cost, and where they ride matters.** A full-size cell out on each flank of every bone, tied softly, to carry the nerves: 0.079 a beat, under half, with its ties stretched to 1.6 times their length. Cells half as wide go 0.112 but their ties stretch to 2.9 and would tear.
- **A nerve cord on top of the bones works.** Two lines of cells 0.7 as wide as a bone, side by side over the middle line as a chordate's nerve cord lies over its rod, each held firmly to its bone (a tie of 100), softly to the next along, and joined to the one beside it: **0.11 a beat, 0.08 a cycle**, no tie past 1.5, and 0.000 in both checks. That is four to thirteen times today's speed with every part in place.
- **The hold across needs no new nerve rule.** The engine already has junctions between tied nerve cells with an inherited weight that can be negative (`development.rs:207`, `neural_contacts`), read once a tick. Modelled that way the two sides still take turns exactly, with 0.24 to 0.83 of a wave on the body for a hold of 0.1 to 0.2. The same junctions learn: cells that fire apart drift toward holding each other down.
- **Length and timing are touchy.** Five bones go 0.031. Thirteen go 0.020 with channels falling 5% and 0.113 with 10%. Both numbers are inherited, so this is for selection to tune, but a starter has to be set inside the range that works.

### What the stroke costs, and how fast the beat should be

The model adds up the work the joints' muscle does and prices it as the engine prices a muscle's work (0.08 of fuel a unit, `WORK_PRICE`). The beat's length is set by how much channel the nerve cells have, which is inherited.

| Joint twist cap | Channels | Beat (ticks) | Lengths a beat | Lengths a cycle | Fuel a cycle | Fuel a body length |
| --- | --- | --- | --- | --- | --- | --- |
| 13 | full | 4.8 | 0.15 | 0.13 | 5.7 | 45 |
| 13 | half | 9.8 | 0.40 | **0.17** | 3.4 | 20 |
| 13 | quarter | 19 | 0.54 | 0.11 | 1.4 | 12 |
| 13 | eighth | 39 | 0.59 | 0.06 | 0.58 | 9.5 |
| 4.4 | quarter | 19 | 0.54 | 0.11 | 0.87 | 7.8 |
| 4.4 | eighth | 39 | 0.59 | 0.06 | 0.38 | 6.2 |
| 1 (today) | quarter | 19 | 0.14 | 0.029 | 0.23 | 8.0 |
| 1 (today) | eighth | 39 | 0.30 | 0.030 | 0.14 | 4.5 |

For scale: grown bodies in last night's ponds took in 0.004 to 0.05 of fuel a cell each cycle, so a body of twenty-odd cells has about 0.1 to 1 a cycle to spend on everything.

- **Fuel is the limit, not mechanics.** At the nerves' present beat the fast stroke costs five to fifty times what a body takes in. It could not have been kept up however good the body.
- **The nerves beat about eight times too fast for a body.** A beat of five ticks is over before the body has bent. At twenty to forty ticks the same body goes three to four times as far each beat (over half its length) for a fifth of the fuel a length. Nerves were made three times faster on 2026-10-12 to get a beat at all; that change works against swimming.
- **A cruise the body can pay for:** a beat of about forty ticks gives 0.03 to 0.06 body lengths a cycle for 0.14 to 0.6 of fuel a cycle. That is two to ten times today's speeds, within what a fed body takes in.
- **A sprint it cannot keep up:** 0.11 to 0.17 a cycle for 0.9 to 3.4 of fuel a cycle, out of stored fat, for a strike or an escape. Real animals split the same way between a cheap cruise and a costly burst.
- **Even today's weak joint cruises** (0.03 a cycle at a twist of 1) once it has a side, a spring and a slow beat. The stronger joint buys the sprint.
- Movement and feeding are one economy: an animal can swim only as fast as it eats. Package 2's gut and size decide how much of the sprint range is ever used.

Not shown on paper: rowing limbs, and how a body would choose between cruise and sprint.

## Shape, linking and fractals

Caden, 2026-10-13: "Shape also plays into dynamics and movement as well, so we need to take that into account as well when creating fins and cell size, specially ones that are linked together", and to look up fractals. Sourced:

| | What the sources say | What it means here |
| --- | --- | --- |
| Drag of linked spheres | A cluster's size grows as the number of spheres to the power of one over its fractal dimension: a chain's with its length, a ball's with the cube root. Its hold on the water is that of a sphere with the same shadow, averaged over directions. Past a hundred spheres the single sphere's size hardly matters. | Law 1 is about the shadow a body casts in the way it is moving, taken for the linked body as a whole. A fin is shadow added one way and not the other. A cell twice as wide has four times the shadow and eight times the weight. The engine's rule that a body is dragged "by how far it reaches" is the direction-averaged case of the same law. |
| Fish | Thrust in a real fish depends on the tail's movement and on the depth of the body there (the water moved goes as depth squared); deep narrow tails give thrust for little drag; a slim body helps only fish that swim with a stiff body. | Depth is shape the engine does not have: its bodies lie flat. A fin stands in for depth and has to be sized as area. |
| Linking | (Paper model above.) | The same cells and the same muscle travel or fold depending on how they are tied. |
| Self-similar bodies | Kriegman and others 2021 built soft robots as robots of robots: every block of a 27-block walker replaced by the whole walker, three levels, up to 19,683 blocks, two muscles on opposite beats. Chosen well, the same shape travels at every size; most fractal shapes do not; evolving one rule for all levels worked and a separate rule for each level was far slower. Shape carried the gait, with one plain beat driving it. | A way to make big animals that needs no new gait: a part of a large animal is itself a small working body. It is also what segments are: one unit that works, repeated. For "parts with real size" (package 2) the choice is between making each cell bigger and making each part a small cluster built like the whole; the second keeps what was tuned. |
| Supply | West, Brown and Enquist: a branching network that fills the body and ends in units of one fixed size gives a metabolic rate that grows as mass to the three-quarter power. Disputed for small bodies and for peak effort. Murray's law: at each fork the cube of the parent's width equals the sum of the cubes of the branches'. | The engine writes three-quarters in by hand for cell size (`pace`, `physical.rs:4751`). The honest route is package 2's circulation: a body supplied through branching vessels with fixed-size ends, and the scaling comes out or it does not. |
| Surfaces | Lungs, gills and the gut's lining are folded on several scales so that surface keeps up with bulk. | A gut or gill is sized by its folded surface, not its volume (package 2). |
| The first big bodies | The Ediacaran fronds (rangeomorphs) were fractal: one branching rule repeated over three or four levels, modelled with 28 numbers, giving tall fronds, flat mats and bundles, all surface, feeding on what is dissolved in the water. They vanish when animals that move arrive. | The step between mats and animals on the timeline. The engine already branches with a taper (`FORK`, `TAPER`), so a frond is a starter the engine can grow now, and movement is what should displace it. |
| Fins and segments | Fin rays are jointed and fork toward the edge. Segments are made one after another from a growing end by a clock. | Fins grow by the same repeat-and-fork rule as fronds. |

## How a frog sets a body up, and what to copy

Caden, 2026-10-13: "lookup like how a frog grows limbs for examples on how we set it up." Until then the bodies in this plan were laid out by hand and tried in the model. Development already has an order and a set of rules for it. Sourced (the signals are known best from chick and mouse; the timing is the frog's):

| Step | How the frog does it | The engine today (`bestiary.rs`, `planned` and `organs`) | Set up the same way |
| --- | --- | --- | --- |
| 1. The body comes first | A tadpole is a rod, a nerve cord on top of it, a muscle block on each flank of every segment, and a tail fin. It swims for weeks before it has a limb. | Every segment cell divides sideways once as it is made, so the swimming line is cluttered with side cells from the start. | Grow the line clean: rod, cord on top, muscle on the flanks, a fin fold. No side branches until step 4. |
| 2. Segments from a growing end | Segments are cut off one after another from the tail end by a clock and a moving front. | Already so: a stem cell at the tip divides and leaves cells behind (`PASSED`, `DIVISIONS`). | Keep. |
| 3. One segment, three tissues, by place | The part of each segment next to the rod becomes skeleton, the outer part a muscle block, the rest skin. Each vertebra is built from the back half of one segment and the front half of the next, so every muscle block is fastened to two vertebrae and crosses the joint between them. | Muscle is put along the middle and bone in every cell; nothing crosses a joint by rule. | Tissue by place across the segment: rod in the middle, muscle outside it, each block tied to the bone ahead and the bone behind. |
| 4. A limb is a bud at a set place, later | A limb starts as a bud of plain cells at one place along the body, hind limbs first. A hormone holds it back until the animal is ready; at a low level limbs grow, at a high level the tail goes. Legs are full size and wired before the tail is lost. | Limbs are made on every segment at once and are at most two cells. | Buds only at set places along the line (the genes already read place, `AXIAL`), and only once the body is grown (genes already read age and fat). |
| 5. A bud grows from its tip | A ridge at the tip keeps the cells under it dividing; cells left behind stop and become what their place says. | The body's own line grows this way. Limbs do not. | A limb is the body's growth rule run again, sideways: its own tip cell, laying a line outward. This is the self-similar body of the fractal work. |
| 6. Base to tip, in order | Upper limb, lower limb, hand: one bone, then two, then many, set by how long each cell stayed under the tip. | Genes read how far out a cell was born (`OUT`). | Bone for the first parts out, a joint between each, a paddle or foot at the end; forking toward the tip as plants already do (`FORK`, `TAPER`). |
| 7. Joints are cut across a rod | Skeleton condenses as rods; a band across a rod turns into a joint, and the cavity opens as the embryo's own muscle flexes it. | A joint forms between any three stiff cells that touch. | A joint only between two bone parts in a line (law 5). |
| 8. Muscle comes in from outside, in two masses | Limb muscle is not made in the limb. Cells leave the body's muscle blocks, walk into the bud and settle as two masses, one each side of the skeleton: the bender and the straightener of each joint. | The joint motor has a bend and a straighten signal but reads one cell. | Two muscle cells at every joint, one each side, each pulling it its own way (law 7's rule, on limbs as on the back). |
| 9. Tendons, then nerves | Tendons form between muscle and skeleton, lengthen as the bone grows, and fail without muscle. Nerves grow in last and wire the two masses separately. | Fibre stiffens a tie along its line; nerve is made wherever a cell has a neighbour. | A muscle's tie to its bone is a fibre tie. Nerves as now. |
| 10. The change to land | Three kinds of change, one hormone: new parts grown (legs, lungs), old parts taken back by cell death (tail, gills), others rebuilt (gut). | No part is ever taken back. | Package 3: a part told to go gives its matter back to the body. |

The tadpole's nerves are also the wiring of law 7, from the animal itself: nerve cells toward the head are more excitable and fire first, the wave passes tailward, and cells that cross the cord hold the other side down (Roberts; Tunstall and Sillar).

**What the model says about step 3.** The frog's own arrangement is a springy rod bent by muscle blocks on its flanks, with no muscle at the joints.

| Slow beat, fin fold on the rod | Body lengths a beat | Each joint's swing, head to tail (radians) |
| --- | --- | --- |
| 9 segments, ties that hold only a length (the engine's) | 0.000 | 0.01 to 0.05: the rod hardly bends |
| 9 segments, each block also held square to the rod | **0.060, head first** | 0.27 0.13 0.16 0.20 0.23 0.21 0.26 |
| The same in old water, or with every nerve cell firing together | 0.000 | |
| 18 segments, a quarter of a nerve wave on the body | 0.033, **tail first** | 0.28 at the ends, 0.04 in the middle |
| 18 segments, nearly two nerve waves on the body | 0.032, head first | 0.28 at the ends, 0.05 in the middle |

- **A tie has to hold an angle, not just a length.** With plain ties the blocks slide along the rod instead of bending it. In the frog a block is fastened to the rod through sheets that cannot slide; in the Cornell robots the beam between two blocks resists bending and shear as well as stretch (Hiller and Lipson). This is very likely the plain cause of every folded body so far, and it is law 6a below. The block is held square to the line through the bones either side of it, so that it does nothing to stiffen the rod.
- **The bend is what geometry says:** about 0.2 radian a joint, from a 22% pull one cell-width off the rod (larval fish muscle is measured to shorten by up to 0.2 as well). How far the tail swings then depends on how slender the body is and how far muscle shortens, not on how many segments it has.
- **In thick water a long body bends only at its ends.** Muscle firing along a long stretch at once asks the whole stretch to swing sideways together, and the water will not let it in the time of a beat. The middle hardly bends, and the wave the body actually makes can run the wrong way, so it swims backward. This is the known rule for a rod in thick water: bending reaches only so far from a free end (Wiggins and Goldstein), and swimmers that live there, sperm and roundworms, carry more than one short wave. Real larvae are not in thick water: they swim where the water's push goes with speed squared (Reynolds 60 to 1,400), with a large swing.
- **So the engine's water is the next question for big animals.** Its drag is in proportion to speed at every size. For anything much bigger than a cell, real water pushes back by speed squared, and that is the water fish and tadpoles are built for. Until it does, a long flank-muscled body will not swim as a tadpole does, and the short bodies above (nine segments, or the line driven at its joints) are what the present water allows.
- In real fish the nerve wave runs down the body faster than the bend it causes (in trout about 4 body lengths a second against 1.7), and muscle near the tail works as a stiff strap more than as an engine. Body stiffness is part of the stroke.

### The same for eggs, gut, supply and feeding (package 2)

Package 2 was also written from first principles. The frog's way, sourced:

| Part | How the frog does it | Set up the same way |
| --- | --- | --- |
| Filling the egg | Yolk is not made in the egg. The liver makes it, the blood carries it, and the growing egg takes it up while it is still in the mother. More eggs or bigger eggs is a trade set by what the mother has; frogs that lay on land or skip the tadpole lay fewer, larger eggs (800 species compared). | The parent fills the egg from its own stores through its circulation while the egg is attached. Egg size is inherited; number follows from what is left. Supply has to exist first. |
| The germ line | The cells that will make the next generation are set aside first, marked by material placed in the egg. | The engine already tracks a germ line; the egg-making cell is that line's, not a limb tip that happens to come off. |
| Cleavage | The egg divides into smaller and smaller cells without growing. Yolkier cells divide more slowly. | A cell may divide without growing while its daughters are still big enough for what they are to be (the rule already noted for eggs). |
| Yolk becomes gut | The yolk-heavy cells become the gut's lining, so the store sits where it will be digested. | The egg's richest cells take the gut fate. |
| Hatching | A tadpole lives on yolk for about a week, starts to feed as it runs out, and can wait some days more. | The hatchling is a complete small body; its mouth works when its yolk is low. No lone-cell larva. |
| The gut | The inner layer rolls into a tube; mouth and vent break through from the skin. Regions follow place along the body (mouth, stomach, intestine). Around the lining go muscle, then a nerve net of its own that walks in from elsewhere. Surface is made by looping and by folds on folds. | A line of lining cells inside the body from mouth to vent, parts by place (`AXIAL`), with muscle around it squeezing in a wave: the swimming wave's machinery again. Sized by folded surface. |
| Supply | Vessels form in place, then sprout toward any tissue short of oxygen, until every cell is within about a tenth of a millimetre of one. | Transport tissue grows toward cells that are short, so the branching net is made by need. This is where the three-quarter scaling should come from, if it comes. |
| Catching food | Tadpoles pump water through the mouth by widening a chamber, and scrape with a horn beak. Fish take prey three ways: suction (widen the mouth fast and the water carries the prey in), ram (swim over it with the mouth open) and biting. Ram does most of the work, because suction and biting reach only a very short way; in some fish over nine-tenths of the suction's power comes from the body's swimming muscle. | Feeding is tied to swimming. A mouth is a chamber opened by muscle (the engine already has muscle-opened mouths and a feeding current); reach comes from the body moving. |
| Jaws | The jaw is the first of the row of gill arches, put to a new use. | Jaws come from a repeated part that already exists (the gill-bearing segments), not from a new organ. |

## How each kind in the cast reproduces

Caden, 2026-10-13: "do research on how reproduction happens for a multitude of organisms we have, each one might be slightly different. I just used a frog as a first example." In the engine today every animal breeds one way: a cell at the end of a limb comes away and has to feed itself up into an egg. Volvox sheds eggs, loose cells ride the wind, and cells can become gametes that fuse. On Earth the cast breeds in about a dozen ways, sourced below.

| Kinds in the cast | How they reproduce on Earth | Rules needed |
| --- | --- | --- |
| bacillus, archaeon, drifter, decomposer, mat, plankton | Split in two. Genes also pass sideways between cells that touch. | Splitting (have). Sideways gene passing is not modelled. |
| cyanobacterium, anabaena | Split. A thread breaks into short moving pieces that start new threads. Some cells give up dividing for good to fix nitrogen for the rest. In bad times a cell thickens its wall and waits (it survives drying and freezing). | Splitting; fragments; resting cell. |
| euglena, paramecium, vorticella, heliozoan, radiolarian, dinoflagellate, swimmer | Split. Paramecium lines wear out over many splits and are renewed only by pairing and swapping nuclei. Dinoflagellates turn to sex when food runs low, and the product is a cyst that lies in the mud for years. Vorticella keeps its stalk and sends the other daughter off as a swimmer. | Splitting; sex under stress; resting cell; a swimming daughter from a fixed parent. |
| diatom | Splits inside a glass box, so one daughter is always smaller. When a line is down to about half size it has sex, and the fused cell swells back to full size and builds a new box. | A shell that cannot grow; sex set off by small size. |
| volvox | A few large cells inside divide into daughter balls. Heat or a signal from males turns the next generation sexual; the fused cell is a resting spore. | Have the first; sex by signal; resting cell. |
| alga, ulothrix, lettuce, seaweed | Two bodies in turn: one makes swimming spores, the other swimming gametes. Sea lettuce releases by the moon. Kelp's sperm only reach eggs about a millimetre away. Often gametes just grow without fusing. | Two-stage life; release in step; gametes must meet. |
| sponge | Sperm into the water, young brooded. Buds. Makes walled balls of cells that survive drying, freezing and no oxygen. | Broadcast; bud; resting body. |
| hydra, coral | Hydra buds a copy off its side and turns sexual when conditions worsen. Corals bud to make the colony, and spawn all together on a few nights set by the moon and the dark after sunset. | Bud (free or attached); release in step. |
| jelly | Egg and sperm make a swimming larva, which settles as a stalked polyp; the polyp slices off small medusae one after another. Two bodies, two ways of breeding. | Two-stage life; bud. |
| dickinsonia | Not known. It grew by adding units at one end. | Growth at one end (have). |
| worm | Sea worms shed eggs and sperm into the water in swarms timed by scent and weather, some turning into swimmers to do it; the young is a ciliated larva. About 1% can regrow from pieces. Roundworms mate, or one animal makes both eggs and sperm. | Broadcast in step; larva unlike the adult; one body with both sexes. |
| trilobite, shrimp, copepod, rower, thornback, grazer, hunter | Eggs. The hatchling is tiny with few segments (a trilobite's 0.1 to 1 mm; a crustacean's with three head segments). It grows only by shedding its armour, and adds a segment or a pair of limbs at most moults until the adult count is reached. | Armour that cannot grow; moulting; segments added at the tail end by stages. |
| scorpion (sea scorpion) | The male set down a packet of sperm and the female took it up and kept it, so she could lay when she chose. They came into the shallows together to moult and mate, and young grew up in separate pools away from fish. | Sperm packet; stored sperm; breeding grounds. |
| dragonfly | Eggs in water. The young lives in water for months to years, moulting, then climbs out and the flier emerges. No resting pupa. | Moulting; two-stage life across water and air. |
| fish, leviathan, courter | Eggs and sperm shed into the water, eggs by the million because nearly all are lost; the hatchling carries a yolk sac and most die in the days after it runs out. | Broadcast; many small eggs; yolk. |
| amphibian | The frog, above. | |
| reptile | Fertilised inside, because sperm cannot swim in air. The egg has a shell (leathery, needing damp), a water bag, a waste bag and a yolk bag. | Mating by contact; the shelled egg. |
| dinosaur, bird | Hard-shelled eggs in a nest, sat on by a parent. Fossils show dinosaurs on their clutches. | The shelled egg; a parent that stays. |
| mammal, ape | Young grow inside, fed through the mother; after birth they are fed milk. Few young. Apes bear one at a time and it stays young longer than in any other mammal its size. | Feeding the young from the body before and after birth; few, costly young. |
| moss | The leafy plant makes sperm that must swim through a film of water to the egg. The fused cell grows a stalk and capsule on the parent and sheds spores to the air. | Two-stage life; sperm that need water; spores on wind (have). |
| fern | The reverse: the big plant sheds spores; each grows a small flat plant on damp ground whose sperm swim to its eggs. | The same, with the other stage large. |
| tree | Pollen on the wind, no water film needed. The seed is an embryo with food and a coat, shed on the wind. | Pollen; seed (a resting stage with stores). |
| flower | Animals carry the pollen. A second fusion makes the seed's food. Fruit gets animals to carry the seed. | The same, carried by animals. |
| grass | Pollen on the wind. Spreads by side shoots and creeping stems into clumps that are all one plant. | Pollen; cloning by shoots. |
| fungus | Threads. Two threads of different mating kinds must meet; the fruiting body lifts spores into moving air and throws them off at over a metre a second. | Mating kinds; spores on wind (have). |

**Twelve rules cover the cast**, and each is built once:

1. Splitting. Have.
2. A resting stage: a walled, stocked cell or body that waits out drying, cold or hunger (akinete, cyst, gemmule, spore, seed). The engine has a dormant state for dried holdfast cells only.
3. Sex set off by hardship or by small size, often making the resting stage. The engine has gametes and fusion; nothing sets them off.
4. Release in step, by a cue all of a kind share (moon, dusk, scent), because gametes shed alone never meet. Not in the engine.
5. Budding: a part grows into a whole new body, which leaves or stays as part of a colony. Not in the engine as a rule; limb ends coming away is the nearest thing.
6. A life in two bodies: larva and adult, polyp and medusa, spore-maker and gamete-maker, with a cue for the change. Not in the engine.
7. Armour that cannot grow, so growth by moulting, with segments added at the tail end by stages. Not in the engine. This is how every arthropod in the cast gets its segments and limbs, as the bud is how the frog gets its legs.
8. A swimming daughter from a fixed parent. Not in the engine.
9. Mating by contact, sperm packets and stored sperm. Not in the engine.
10. The stocked egg, from yolk sac to shell to nest (the frog section above).
11. Feeding the young from the body: inside, and by milk. Not in the engine.
12. Wind: spores (have), pollen and seeds.

What this changes: package 2's "eggs" was one design for every animal. It becomes these rules, built in the order the timeline needs them. Seas first: resting stage, sex under stress, release in step, budding, two-body lives, moulting. Then land: contact mating, pollen and seed, the shelled egg. Then care and milk.

## The laws to write

Each replaces a shortcut or a cap with a rule life has to work with.

1. **Drag by shape.** The water holds a body by the shadow it casts in the direction it is moving. A part is shielded along the line of its ties and exposed across it: half the drag lengthwise for a part in the middle of a straight line (the thin-filament value), no change for a free cell or a compact ball, and the body's total broadside drag as now. A fin widens a part across its ties and adds nothing along them. A body three cells wide and nine long has little of this by itself (about 1.3 to 1 on paper); it needs fins or depth.
2. **No one-way grips.** `fin_drag`, the foot's dig-in and (at the flight stage) the wing's back-stroke grip are removed.
3. **Friction by load.** A part on the ground holds until pushed past its friction limit, then slides against the same limit, in every direction.
4. **Bone is a part.** A part given over to bone is mostly mineral. A tie between two bone parts keeps its length against any muscle. Stiffness is measured against dry tissue. The 15% cap goes: how much bone a body carries is settled by the mineral it can find, by sinking in water (bone is 2.5 times water's density already) and by weight on land.
5. **Joints along a limb, with stops, a side and a spring.** A joint forms where two bone ties meet end to end, one for each such meeting. It knows left from right. It has a spring of its own toward its resting shape (2 on paper), which muscle works against and which no cap removes. Its muscle twists it as hard as the same muscle pulls a tie one cell-width away (13, not 1), and takes time to pull it round (half-way each tick). It is stiff enough to follow a beat (60). It swings a set range and then stops hard, bone on bone (0.9 radian, under the one radian at which a short swimmer runs backward). Its pushes on its three cells are held back together or not at all, so they always sum to nothing.
6a. **A tie holds an angle.** Tissue fastened to a rod stays square to it: a tie between a stiff part and its neighbour resists being swung, as well as being stretched. (On paper: each flank cell held square to the rod at its own bone, stiffness 10.)
6. **A body does not glue to itself.** Parts tie only to the parts development set beside them, not to whatever they later touch.
7. **The head end is quickest, the sides take turns, and the beat is slow.** Every nerve cell keeps its own beat, as now, but with an eighth to a quarter of the channel a starter has today, for a beat of twenty to forty ticks. Channels fall a little from head to tail (5 to 10%), set by the body-plan genes that already read a cell's place along the body, so each cell trails the one ahead. The two lines of nerve cells ride on the bones as a cord, and each cell holds down the one beside it through a junction of negative weight, which the engine already has. The one new rule: the joint motor bends one way on the firing of the nerve cell on that side and the other way on the firing of the cell across (today it reads one cell). The tie strengths that work on paper (0.05 to 0.10) are inside what the conduction gene already gives. This is the lamprey's arrangement: a chain of segments each with its own rhythm, the more excitable end leading.
8. **Water evens out** between joined parts in proportion to the difference, as fuel will through the gut stage's circulation.

## Build order

Three packages. Each is written whole, then built once and checked once.

**Package 1: movement.** Laws 1 to 7, then the bodies, all before any run.
- Bodies are grown in the frog's order (see "How a frog sets a body up"), not laid out by hand: the swimming line first, limbs later from buds.
- Swimmers by bending (Worm, Fish, Eel, Amphibian's trunk): a single-file middle of about nine parts jointed end to end, with a fin on every part or at least the last two, and a cord of two lines of smaller nerve cells riding on it, pulling each joint from its two sides. The Worm's middle is stiffened by pressure and fibre, the others' by bone. Whether the muscle sits at the joints (which travels on paper) or in blocks on the flanks as in a tadpole (which does not yet) waits on ties that hold an angle.
- Rowers (Trilobite, Shrimp, Sea scorpion): a stiff middle; each limb two bone parts and a paddle, the two joints a quarter-beat apart so the paddle comes forward folded; limbs out of step from back to front.
- The Jelly rows too. A jet works only in thin water (simulated bells lose their speed below a Reynolds number of about 10, and almost nothing that jets swims below 5); in thick water a young jellyfish's arms act as one paddle. So the bell's rim is a ring of paddles, spread on the stroke and folded on the return.
- Everything that swims by cilia is untouched.

How fine a body has to be (sourced): three stiff links on two hinges moved out of step is the least that can swim in thick water (Purcell's swimmer), and for a body of many links the cheapest stroke is a travelling wave of bending whose shortest useful wave is three links long. Nine parts carry a wave comfortably. A three-link swimmer swung past about one radian goes backward, so joint stops are set under that.

Gates, in order, each a hand-built fixture that runs in well under a second:
1. A straight rod towed lengthwise meets half the drag it meets broadside (within a tenth). A ball meets the same both ways.
2. A body that opens and shuts travels nowhere over twenty beats. (Today it travels. This test is the proof the one-way grip is gone.)
3. A hand-built line of nine bones with a fin on each and a nerve cord on top, wired by law 7 and jointed by law 5, travels 0.11 body lengths a beat, head first, within a quarter (the paper model's figure), and nothing with every nerve cell firing together.
4. A two-bone hinge holds its stop under full muscle. A line of bone parts driven at its joints bends and does not shorten more than 2%.
5. Nerves alone, no water: a nine-segment ladder wired by law 7 keeps every cell in step for 200 beats, left and right half a beat apart within a twentieth, and between 0.4 and 0.9 of a wave on the body, head to tail.
6. Then, once, `probe_stroke` for each grown swimmer and rower, fed and paying for its own stroke (the probe today refills fuel every tick, which hides the cost): 0.03 body lengths a cycle or better kept up for 400 cycles on what it eats, no joint past its stop, no tie stretched past 1.7, the body whole at the end. That is about twice today's best and five times its worst, at a price the body can pay.

Tests that assert travel by fins or feet under the old rule will fail on purpose and are rewritten to the new law.

**Package 2: a real animal.** Only after package 1's gate 6. Set up in the frog's order (see "The same for eggs, gut, supply and feeding"): supply first, then gut, then eggs the parent fills.
- Size: parts many times a plankton cell's bulk; tie and muscle strength by cross-section (today a tie stops getting stronger above a mass of 0.25, `SLIGHT`); hunters 10 to 100 times their prey.
- Gut: prey swallowed whole into a hold of 15 to 30% of the body, digested over time, 60 to 90% of flesh absorbed and less of plants, the rest passed as remains; the mouth free at once (`HANDLE` goes).
- Circulation: fuel and water even out between joined parts (law 8).
- Eggs: the parent fills an egg part from its surplus while it is attached; let go when full; it divides down without growing into a complete small body, which feeds when the yolk is gone. Egg size is inherited, so many-small against few-large is settled by survival.
- One organism: a part torn off dies unless it is an egg (the lone-cell larval mouth goes); the app counts animals.

Gate: in the fed pond, a grown hunter's intake at least twice what it burns over its first 200 cycles, second-generation bodies by cycle 400, and no loose stalled young.

**Package 3: land and air.** Only after package 2's gate.
- Legs come from the water (sourced): the first four-limbed animals were swimmers with limbs (Acanthostega kept gills and could not hold its chest up out of water), and Tiktaalik propped itself on its fins on stream beds. So a leg is a rower's limb used on the bottom, and the first walker is a bottom-walker in the shallows. No new limb is designed for land.
- Walking: a leg in swing is lifted and bears nothing; a leg in stance holds by friction (law 3) and the body vaults over it. Legs move in a wave from back to front at low speed. The Amphibian keeps its swimming wave: a salamander's trunk bends as it walks, and one held stiff bends less and loses stride. Physics sets the top walking speed without a rule: speed squared over gravity times leg length (the Froude number) cannot pass 1 in a walk, and animals change gait near 0.5.
- Weight: bone carries it (law 4); a body too heavy for its legs is crushed, as now.
- Water: lost through bare skin, kept by a waxy cuticle, fastest from small bodies; an egg laid on land needs a shell that passes gas and keeps water, with yolk inside (the Carboniferous egg). The first land animals were arthropods (about 420 million years ago), which breathe through inward pockets and tubes that lose little water. Whether the engine charges breathing in air against drying was not checked and is the first thing to read.
- Flight: law 2 applied to wings, and law 1 in air. A wing is a paddle that turns over at each end of its stroke (insects twist the wing at stroke reversal to set its angle), so it is the rower's two-joint stroke with lift already in the engine. Check against the measured rule that wingbeat rate goes as the square root of body mass over wing area.

Gates: a bottom-walker crosses a dry bar between two pools; a walker's speed stays under a Froude number of 1; an egg with a shell hatches on dry ground and one without dries; a flier holds height for 100 cycles with no one-way grip, at a beat rate within a third of the rule.

**Then** the planet runs for the timeline (Devonian onward), which is what the three packages are for.

## Testing budget

- No build, test, probe or run without Caden's word. Code for a whole package is written first.
- When a build is agreed: low priority and two cores (`nice -n 19 cargo test --release -j 2 --lib <name> -- --test-threads=1`), new tests by name only. The whole suite and the check scripts in `tests/` run once at the end of a package.
- Pond probes: once per package, at its last gate. Planet runs: only after package 2, overnight or when he is away.
- A gate that fails is diagnosed from its own small fixture, not by a larger run.

## Not known

- The paper model gives the joint angles and lets the water answer. It does not show that the engine's joints, ties and muscle will hold that shape against the water; gates 3 and 4 do.
- Rowing limbs were not modelled beyond the three-link swimmer. Their timing (two joints a quarter-beat apart) is from that and from the sources on krill, not from a worked case.
- The Cornell figure for distance: the chart's axis runs to about 4 body lengths in ten beats; the exact champion values were not read.
- That in the lamprey the most excitable segment leads the wave (Matsushima and Grillner 1992) was recalled, not found in the searches. Law 7 rests on the paper model, which shows the quicker end leading in the engine's own equations.
- Law 6 rests on general biology (sheets of skin keep tissues apart), not on a source searched here.
- Whether a walk by vaulting applies to sprawling four-legged animals: the sources for it are about two-legged walkers.

## Sources

Read in full or in the relevant part:
- Sims, "Evolving Virtual Creatures", SIGGRAPH 1994: https://www.karlsims.com/papers/siggraph94.pdf
- Cheney, MacCurdy, Clune, Lipson, "Unshackling Evolution", GECCO 2013: http://jeffclune.com/publications/2013_Softbots_GECCO.pdf
- Hiller and Lipson, "Dynamic Simulation of Soft Heterogeneous Objects": https://arxiv.org/abs/1212.2845

From search summaries (not the papers themselves):
- Low-Reynolds-number swimming and resistive force theory: https://arxiv.org/pdf/1410.4321 ; flagellar propulsion: https://cob.silverchair.com/jeb/article/209/6/985/16571/FLAGELLAR-PROPULSION
- Fish muscle and body waves: https://research.wur.nl/en/publications/undulatory-fish-swimming-from-muscles-to-flow ; https://sites.harvard.edu/glauder/files/2022/03/JayneLauder1996.pdf ; stiff column slows the wave: https://sicb.org/?p=27422 ; timing and thrust: https://arxiv.org/pdf/2512.01218v1.pdf
- Lamprey phase lag and crossed inhibition: https://link.springer.com/article/10.1007/s004220050565 ; https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5990913/ ; https://cob.silverchair.com/jeb/article-split/112/1/337/4127/How-does-the-Lamprey-Central-Nervous-System-make
- Hydrostatic skeletons: https://cob.silverchair.com/jeb/article/215/8/1247/11327/The-diversity-of-hydrostatic-skeletons ; opposed muscles: https://digfir-published.macmillanusa.com/morris2e/morris2e_ch37_12.html
- Skeleton share of body mass: https://people.ohio.edu/witmerl/Downloads/2014_Eastman_et_al._Antarctic_fish_skeletal_divergence.pdf ; https://bionumbers.hms.harvard.edu/bionumber.aspx?id=108636 ; arthropod shells: https://sicb.org/?p=29100
- Joint stops: https://spiral.imperial.ac.uk/entities/publication/d7690671-8f88-460d-8d12-7a51ba499f82
- Insect gaits: https://doaj.org/article/7bed1e0f7a874227bfb99be04cab5570 ; wing loading: https://www.biorxiv.org/content/10.1101/2022.06.09.495408v1.full.pdf
- Guts as reactors and gut size: https://www.journals.uchicago.edu/doi/10.1086/284623 ; https://digital.lib.washington.edu/researchworks/items/a111c044-f25b-4db7-a193-3fa9af8d92db
- Predator and prey size: https://epic2-clone.awi.de/15076/1/Brose06_Ecology87.pdf
- Egg size and number: https://link.springer.com/article/10.1007/s00442-007-0952-3 ; https://academic.oup.com/book/39876/chapter/340063816 ; yolk and first feeding: https://orbit.dtu.dk/en/publications/first-feeding-of-larval-herring/ ; direct development: https://pmc.ncbi.nlm.nih.gov/articles/PMC5102429
- Water on land: https://peerj.com/articles/2382 ; https://gibbs.faculty.unlv.edu/JIP%20Scorpion%202009.pdf
- First animal movement: https://www.ox.ac.uk/node/8412 ; Cambrian swimmers: https://www.sciencenews.org/article/devonian-swimming-revolution-oceans-may-have-never-happened
- Self-similar robots: https://arxiv.org/abs/2103.04876 ; drag of clusters of spheres: https://authors.library.caltech.edu/records/2f4nb-xj544 ; https://ideas.repec.org/a/eee/matcom/v72y2006i2p103-107.html
- Branching supply and three-quarter scaling: https://pdodds.w3.uvm.edu/teaching/courses/2009-08UVM-300/docs/others/1999/west1999a.pdf ; its limits: https://fontana.hms.harvard.edu/sites/fontana.hms.harvard.edu/files/documents/finite_size_scaling.combined.pdf ; Murray's law: https://arxiv.org/pdf/1812.09706
- Fractal fronds of the Ediacaran: https://pmc.ncbi.nlm.nih.gov/articles/PMC4246981 ; https://en.wikipedia.org/wiki/Rangeomorph
- Fish shape and thrust: https://arxiv.org/pdf/2409.20162 ; https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3799785/ ; fin rays: https://www.biorxiv.org/content/10.1101/2022.05.09.491182.full.pdf
- How a frog builds limbs and body: limb signals: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6027391/ ; hind limbs first and the hormone that times them: https://pmc.ncbi.nlm.nih.gov/articles/PMC1467782 ; https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8835992/ ; limb parts and digits: https://en.wikipedia.org/wiki/Limb_development ; https://pmc.ncbi.nlm.nih.gov/articles/PMC4216602 ; joints: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9212078/ ; limb muscle from the body's blocks: https://pmc.ncbi.nlm.nih.gov/articles/PMC1571050/ ; tendons: https://cob.silverchair.com/dev/article-split/128/19/3855/41388/Analysis-of-the-tendon-cell-fate-using-Scleraxis-a ; segments and vertebrae from half segments: https://en.wikipedia.org/wiki/Somitogenesis ; https://link.springer.com/article/10.1007/s004290000110 ; tadpole swimming nerves: https://pmc.ncbi.nlm.nih.gov/articles/PMC3970992 ; https://cob.silverchair.com/jeb/article/99/1/175/23179/The-neuromuscular-basis-of-swimming-movements-in ; https://research-portal.st-andrews.ac.uk/en/publications/physiological-and-developmental-aspects-of-intersegmental-coordin/ ; the change to a frog: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6426756/
- Muscle blocks and the sheets between them: https://en.wikipedia.org/wiki/Myomere ; https://link.springer.com/article/10.1186/2041-9139-3-12 ; nerve wave faster than the bend: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC2518218/ ; https://arxiv.org/pdf/1812.02410 ; https://cob.silverchair.com/jeb/article/143/1/559/5978/Locomotion-in-Lamprey-and-Trout-the-Relative ; how far bending reaches in thick water: https://arxiv.org/pdf/cond-mat/9707346 ; larval fish strain and Reynolds number: https://cob.silverchair.com/jeb/article-pdf/207/5/853/1251455/853.pdf ; https://pmc.ncbi.nlm.nih.gov/articles/PMC4614456
- Frog egg to feeding tadpole: https://embryo.asu.edu/pages/gastrulation-xenopus ; https://pmc.ncbi.nlm.nih.gov/articles/PMC5312736 ; yolk made in the liver: https://en.wikipedia.org/wiki/Vitellogenesis ; egg size against number in 800 amphibians: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8726499/ ; germ line set aside: https://pmc.ncbi.nlm.nih.gov/articles/PMC4550962 ; gut tube, regions, folds and its nerves: https://pmc.ncbi.nlm.nih.gov/articles/PMC2861293 ; https://pmc.ncbi.nlm.nih.gov/articles/PMC8206791 ; vessels grown toward need: https://pubmed.ncbi.nlm.nih.gov/10474163 ; tadpole pumping: https://link.springer.com/doi/10.1007/BF00345438 ; suction, ram and biting: https://experts.nau.edu/en/publications/suction-ram-and-biting-deviations-and-limitations-to-the-capture-/ ; jaws from gill arches: https://stemcell.keck.usc.edu/how-did-vertebrates-evolve-jaws/
- Lancelet germ line and spawning: https://pmc.ncbi.nlm.nih.gov/articles/PMC3735472 ; https://www.frontiersin.org/journals/ecology-and-evolution/articles/10.3389/fevo.2015.00156/full ; https://cshprotocols.cshlp.org/content/2009/9/pdb.prot5285.abstract ; https://link.springer.com/doi/10.1007/BF02850488 ; https://pubmed.ncbi.nlm.nih.gov/23990962/
- Reproduction across the cast: cyanobacteria: https://www.frontiersin.org/journals/microbiology/articles/10.3389/fmicb.2021.645028/pdf ; Paramecium: https://en.wikipedia.org/wiki/Paramecium_caudatum ; https://en.wikipedia.org/wiki/Autogamy ; diatoms: https://rbg-web2.rbge.org.uk/algae/auxospores/LifeCycle_auxospore.html ; https://www.whoi.edu/home/interactive/Pseudonitzchia-LifeCycle-master/Pseudonitzchia-LifeCycle-master.html ; Volvox: https://en.wikipedia.org/wiki/Volvox_carteri ; dinoflagellates: https://www.whoi.edu/home/interactive/Alexandrium-LifeCycle-master/Alexandrium-LifeCycle-master.html ; sea lettuce and kelp: https://www.seaweed.ie/algae/ulva.php ; https://arxiv.org/pdf/2404.14003 ; mosses and ferns: https://people.ohio.edu/braselto/pdfs/HowDoPlantsDiffer.pdf ; sponges: https://en.wikipedia.org/wiki/Gemmule ; https://pmc.ncbi.nlm.nih.gov/articles/PMC2749020 ; jellyfish: https://www.vims.edu/bayinfo/jellyfish/lifecycle/ ; https://thescyphozoan.ucmerced.edu/Biol/Ecol/LifeHistory/ScyphozoaLH.html ; coral spawning: https://pmc.ncbi.nlm.nih.gov/articles/PMC8403928 ; https://researchonline.jcu.edu.au/52181/ ; trilobite growth: https://www.biologia.unipd.it/giuseppe.fusco/PDFs/2006%20Hughes%20et%20al%20Paleobiology.pdf ; https://www.frontiersin.org/journals/ecology-and-evolution/articles/10.3389/fevo.2021.622482/full ; crustacean larvae: https://en.wikipedia.org/wiki/Crustacean_larva ; dragonflies: https://www.amentsoc.org/insects/fact-files/orders/exopterygota.html ; sea scorpions: https://bathgeolsoc.org.uk/lectures/2001-04-05-giant-sea-scorpions ; https://wp.unil.ch/paleo/?p=637 ; worms: https://books.byui.edu/Invertebrate_Life/anhrmxehll ; https://pmc.ncbi.nlm.nih.gov/articles/PMC4199697 ; fish spawning: https://en.wikipedia.org/wiki/Spawning ; https://arxiv.org/pdf/1603.04007 ; shelled eggs: https://courses.lumenlearning.com/suny-wmopen-biology2/chapter/amniotes/ ; dinosaurs on nests: https://www.nhm.ac.uk/discover/were-dinosaurs-good-parents.html ; mammals: https://pmc.ncbi.nlm.nih.gov/articles/PMC2892970 ; primates: https://link.springer.com/article/10.1007/s10329-006-0193-9 ; conifers: https://conifers.org/topics/seed-dispersal.php ; flowers: https://bio.libretexts.org/Bookshelves/Introductory_and_General_Biology/General_Biology_(Boundless)/32:_Plant_Reproductive_Development_and_Structure/32.07:_Pollination_and_Fertilization_-_Double_Fertilization_in_Plants ; fungi: https://digfir-published.macmillanusa.com/hlw/hlw_ch34_3.html
- Fewest links that swim, and the wave as the cheapest stroke: https://arxiv.org/pdf/1603.02268 ; https://ar5iv.labs.arxiv.org/html/1801.04687 ; https://www.princeton.edu/~stonelab/Publications/pdfs/From%20Howard/JFM/BeckerKoehlerStoneJFM03.pdf
- Jellyfish: jets fail in thick water: https://experts.arizona.edu/en/publications/reynolds-number-limits-for-jet-propulsion-a-numerical-study-of-si/ ; young jellyfish paddle: https://ar5iv.labs.arxiv.org/html/1010.3357
- Walking and the Froude number: https://pmc.ncbi.nlm.nih.gov/articles/PMC1617162 ; salamander trunk and limbs: https://sicb.org/?p=39568 ; https://cob.silverchair.com/jeb/article-pdf/193/1/285/2854976/jexbio_193_1_285.pdf
- Limbs before land: https://evolution.berkeley.edu/evolibrary/news/060501_tiktaalik ; the shelled egg: https://en.wikipedia.org/wiki/Amniote ; first land arthropods: https://www.nature.com/articles/384555a0 ; https://pmc.ncbi.nlm.nih.gov/articles/PMC7657913
- Insect flight: https://cob.silverchair.com/jeb/article-pdf/207/7/1137/1251682/1137.pdf ; wingbeat rule: https://phys.org/news/2024-06-frequency-birds-insects-whales-body.html
- Swimming speed and muscle share: https://doi.org/10.2331%2Fsuisan.59.745 ; https://www.frontiersin.org/journals/physiology/articles/10.3389/fphys.2014.00506/pdf
