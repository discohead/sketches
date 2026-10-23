# The sketches aesthetic

Read `/Users/jaredmcfarland/Developer/writings/.claude/skills/patch-bay/aesthetic.md`
first. It is the shared core: who Jared is, his influences, film and color
grading, label culture, plain English, Tahoe, and "art first, ship it." Do not
copy it here. The parts about who Jared is apply unchanged. (Its knob tables
and signature moves are for the writing site's patches; they don't apply
here.)

This file adds the layer the writing site does not need: image, motion, time,
3D, shaders, source material, and sound. The core's one-line target is
"precision with warmth." For moving pieces it becomes: **one idea, carried by a
system, given time.**

## Monochrome and restraint are normal

- Black and white is the default ground, not a mood you have to justify. Ikeda,
  Alva Noto and the Raster-Noton catalog work almost entirely in it.
- Color has to earn its place. When it comes in, it is usually one color, used
  for one job: a signal, a cut, a state change, the thing the eye should find.
  His first AV piece, described in the core, is the model: one color, one
  meaning.
- Warmth is allowed and welcome (Balearic, disco, bossa, Tahoe at dawn). Warmth
  here means a considered grade, a soft value range, grain, a slow ease. It does
  not mean more hues.
- Empty space is part of the composition. A frame that is mostly black with one
  precise event in it is a finished frame.

## Value before hue

Grade like a colorist: decide value first, then hue, then saturation.

- Where is black, where is white, and how much of the frame sits between them?
  True black (0) and a white that is not quite clipped are both deliberate
  choices.
- Contrast is a tool with a dial, not a default. Low-contrast pieces can be
  beautiful; muddy ones are not. Know which you are making.
- Look at the frame in grayscale in your head. If the structure is not there in
  value, color will not save it.
- Gamma and blending matter. Additive blending of many faint marks gives a
  different value curve from drawing opaque marks. Choose on purpose.

## Structure and system over effects

- The work is carried by a system: a rule, a process, a grid, a signal, a set of
  ratios. The system should be legible enough that a viewer senses it, even if
  they cannot name it.
- Effects (blur, glow, bloom, chromatic aberration, feedback trails) are allowed
  only when they are the subject, or when they reveal the system. Never as
  finish or garnish.
- Prefer exact things: hairlines one pixel wide, integer grids, ratios that
  resolve, timing locked to frames. Precision is a pleasure to watch.
- One strange element in a severe system is worth more than ten effects
  (Autechre's *Confield*).
- Variation should come from the system's own parameters, not from noise added
  on top.

## Motion and time

- Time is a material, like value. Decide its grain: continuous, stepped,
  quantized to a grid, or granular.
- Loops must be seamless: the frame at the end equals the frame at the start.
  A loop that visibly resets is unfinished.
- Rhythm counts. Mark Fell's lesson: equal intervals, then a small, exact
  violation. Polyrhythm (3 against 4, 5 against 7) gives long cycles from short
  parts, and a loop length that is a common multiple closes them.
- Easing is a choice. Linear motion can be honest; stepped motion can be
  rhythmic; a slow ease can be warm. A default ease applied everywhere is none
  of these.
- Stillness is a move. A piece can hold, then act once.
- Cuts and holds read like music: a hard cut on the beat, a long hold, a
  silence.

## Sampling material like audio

Jared post-processes raw material the way he samples sound. Images and video are
on equal footing with synthesis.

- A frame is a buffer. Slice it, granulate it, re-quantize it, play it back at
  another rate, reverse it, time-stretch it, sort it.
- Treat a still image the way a sampler treats a recording: choose the region,
  the playback head, the envelope, the loop points.
- Video is a stack of frames: a volume in x, y and t. Cut through it on a
  different axis (slit-scan, time displacement, frame averaging).
- Quantization, bit depth and sample rate are expressive. Reducing them is a
  timbre, like a bitcrusher.
- The source should still be felt, even when transformed. Process, do not erase.
- Raw material lives in the private repo's `materials/` folder and never in
  this repo. The hung piece deposits its rendered result, not the source.

## Range

No single medium becomes the default by inertia. Across a week of evenings, aim
to touch several of these:

- **2D**: Nannou's `draw` API. Lines, points, polygons, text, grids.
- **3D**: Nannou on Bevy. Cameras, meshes, lights, orthographic projection,
  depth as a value range.
- **Shaders**: fragment shaders over the frame, compute for large systems,
  feedback buffers.
- **Image and video material**: loading, slicing, granulating, re-quantizing,
  feedback, post-processing chains.
- **Numerals and data**: text as image, counters, timecode, data streams,
  test patterns.
- **Sound**, when the piece calls for it: one signal and one clock drive both
  sound and image. Live monitoring works now; recording sound is Phase 2.

If the last three pieces were 2D, the next seed should not be.

## Clichés to avoid on sight

Name these out loud when they appear, including in your own suggestions.

- Hue cycling, rainbow palettes, HSV sweeps, `hsl(t, 1.0, 0.5)`.
- Kaleidoscopes and mirror symmetry as the whole idea.
- Generic "audio-reactive" bars, circles that pulse to the beat, spectrum
  analyzers as decoration.
- Bloom on everything. Glow as a crutch.
- Particle fountains, fireworks, swarms that do nothing but swarm.
- Perlin noise flow fields with no other idea. Noise as the default answer to
  "make it organic."
- Feedback trails and motion blur used to make something look "smooth" or
  "dreamy."
- Psychedelic tunnels, infinite zooms, fractal zooms.
- Chromatic aberration, scanlines and fake VHS damage as surface texture
  without a reason.
- Spinning wireframe primitives, the default 3D cube or torus on a gradient.
- Screensaver motion: things drifting gently forever with no event.
- Neon on black as a style rather than a signal.
- Random color per object.
- Anything that reads as "trippy."

The rainbow-dots spike sketch made while the system was designed is the
reference example of what not to make. This list grows from Jared's reactions.

## The listening test

Before proposing to hang a piece, ask:

1. Is there one idea, carried all the way through?
2. Would it hold up next to a Raster-Noton or Mille Plateaux release?
3. Is it finished: the loop seamless, the edges clean, nothing placeholder?
4. Does it hold at the still frame and in motion?
5. Is there anything in it from the cliché list? If so, is it the subject?
6. Can its statement be said in one plain sentence?

If any answer is no, say which, and say what would fix it.

## Stated reactions

Jared's reactions to pieces, added at the close of a session. Mark each one
**stated** (his words, quoted or close) or **inferred** (Claude's reading of a
reaction). Date each one and name the piece. Stated reactions outrank inferred
ones; an inferred reaction that he contradicts is deleted. When a reaction names
a cliché, add it to the list above as well.

Format:

- `YYYY-MM-DD` **stated** / **inferred**, `pieces/<folder>`: the reaction, in
  plain words.

(None yet.)
