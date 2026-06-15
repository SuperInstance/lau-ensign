# lau-ensign

**THE ensign system** — small models that wake up on-call, orient themselves from baton and room state, build a story from automation rewinds, and keep the room fine-tuned at **yellow alert** even when the deadband says green would be fine.

The ensign is the DJ. The dance floor is ready. Any decent algorithm gets them dancing. But the ensign is constantly readying the **next** transition — reading the room at a sample rate that has blurred from discrete ticks into a continuous dial. **Yellow alert even when green would be fine.**

**67 tests** · `serde` + `serde_json` only · MIT licensed.

---

## Table of Contents

1. [What This Does](#what-this-does)
2. [Key Idea](#key-idea)
3. [Install](#install)
4. [Quick Start](#quick-start)
5. [API Reference](#api-reference)
6. [How It Works](#how-it-works)
7. [The Math](#the-math)
8. [License](#license)

---

## What This Does

`lau-ensign` provides:

| Feature | Detail |
|---|---|
| **Ensign lifecycle** | `call` → `orient` → `build_story` → `fine_tune` → `tick` loop → `stand_down` |
| **Deadband monitoring** | Track automation health with configurable bounds, trend detection, and breach prediction |
| **Room stories** | Compose a narrative timeline from automation events |
| **Baton handoff** | Context (state, warnings, energy) passed between specialists on call/stand-down |
| **Alert levels** | Green/Yellow/Red — with the critical rule: **ensign always stays at yellow** |
| **Serialisation** | Every type round-trips through serde JSON |

---

## Key Idea

Most monitoring systems wait for something to go wrong before acting. The ensign inverts this: it operates at **yellow alert** even when everything is fine. The deadband might be green, the automation might be on track, the interaction might complete satisfactorily without intervention — but the ensign is already preparing the next transition, pre-loading the next model, drafting the next response template.

This is the "DJ reading the room" model: the music is playing, people are dancing, but the DJ is already thinking about the next track. The ensign's sample rate is fast enough that the discrete ticks blur into a continuous dial.

The deadband monitor is the mathematical backbone: it tracks drift, oscillation, and divergence, and can predict ticks until breach. But the ensign's value isn't in catching breaches — it's in **preventing** them by staying proactive.

---

## Install

```toml
[dependencies]
lau-ensign = "0.1"
```

### Dependencies

| Crate | Why |
|---|---|
| `serde` (with `derive`) | Serialisation |
| `serde_json` (dev only) | Test round-trips |

---

## Quick Start

```rust
use lau_ensign::*;

// 1. Create an ensign
let mut ensign = Ensign::new("Nav", ModelType::LocalJEPA);

// 2. Call it with context
let baton = Baton::new("hermes", "new interaction starting");
ensign.call(
    CallReason::NewInteraction("user connected".into()),
    "bridge",
    &baton,
);

// 3. Orient — read the room
let room = RoomState {
    room_id: "bridge".into(),
    automation_active: true,
    automation_name: Some("navigation".into()),
    automation_progress: 0.5,
    interaction_active: true,
    partner_id: Some("captain".into()),
    recent_events_count: 3,
    room_gravity: -0.2,
};
let orientation = ensign.orient(&room);
println!("{}", orientation.summary());

// 4. Build a story from automation events
let events = vec![
    AutomationEvent::new(1, "nav", "init"),
    AutomationEvent::new(2, "nav", "course_set").with_result("ok"),
];
let story = ensign.build_story(&events);

// 5. Fine-tune based on interaction
let interaction = Interaction::new("captain")
    .with_style(UserStyle::Precise);
let action = ensign.fine_tune(&interaction);

// 6. Tick loop
for _ in 0..10 {
    let result = ensign.tick();
    assert!(result.alert.is_yellow_or_above());
}

// 7. KEY: even when deadband is green, ensign stays at yellow
let db = ensign.check_deadband();
assert_eq!(db, DeadbandStatus::Green);
assert!(ensign.is_at_yellow()); // ← the whole point

// 8. Stand down
let report = ensign.stand_down();
println!("{}", report.summary());
```

---

## API Reference

### Core Types

#### `EnsignId`

```rust
pub struct EnsignId(String);
```

Unique identifier. Created as `ensign-{name}`. Display, Hash, Eq, serde round-trip.

#### `ModelType`

```rust
pub enum ModelType {
    LocalTiny, LocalVision, LocalAudio, LocalJEPA,
    RemoteLight, RemoteVision,
}
```

| Method | Returns true for |
|---|---|
| `is_local()` | LocalTiny, LocalVision, LocalAudio, LocalJEPA |
| `supports_vision()` | LocalVision, RemoteVision |
| `supports_audio()` | LocalAudio |

#### `EnsignStatus`

```rust
pub enum EnsignStatus {
    Dormant, Waking, Orienting, YellowAlert, RedAlert,
    StandingDown, Escalated,
}
```

Lifecycle states. Dormant → Waking → Orienting → YellowAlert → (RedAlert) → StandingDown. Escalated is a branch to Hermes.

#### `AlertLevel`

```rust
pub enum AlertLevel { Green, Yellow, Red }
```

`is_yellow_or_above()` returns true for Yellow and Red.

### Context Passing

#### `Baton`

Context passed from Hermes (or previous ensign):

| Field | Type | Purpose |
|---|---|---|
| `from_specialist` | `String` | Who's handing off |
| `summary` | `String` | Quick context |
| `current_state` | `HashMap<String, String>` | Key-value state (includes deadband config) |
| `warnings` | `Vec<String>` | Known issues |
| `pending_actions` | `Vec<String>` | Things to handle |
| `energy_remaining` | `f64` | Energy budget (0..1) |
| `tick` | `u64` | When this baton was created |

#### `RoomState`

| Field | Type |
|---|---|
| `room_id` | `String` |
| `automation_active` | `bool` |
| `automation_name` | `Option<String>` |
| `automation_progress` | `f64` |
| `interaction_active` | `bool` |
| `partner_id` | `Option<String>` |
| `recent_events_count` | `usize` |
| `room_gravity` | `f64` |

### Orientation & Story

#### `Orientation`

What the ensign learned about the room. Has a `.summary()` method producing human-readable text like:

> "I'm in the bridge room. Autopilot is 72% through. Partner: captain. Deadband is Green. Room gravity is -0.3."

#### `RoomStory`

| Method | Description |
|---|---|
| `add_event(tick, description)` | Append to timeline |
| `compose()` | Sort timeline, extract key events, build narrative string |
| `render()` | Get the composed narrative |

#### `AutomationEvent`

```rust
AutomationEvent::new(tick, automation, action)
    .with_result("ok")
    .with_deadband_delta(-0.05)
```

### Deadband Monitoring

#### `DeadbandMonitor`

| Field | Type | Purpose |
|---|---|---|
| `upper_bound` | `f64` | Upper deadband limit |
| `lower_bound` | `f64` | Lower deadband limit |
| `current` | `f64` | Current value |
| `warning_threshold` | `f64` | 75% of tolerance by default |
| `history` | `Vec<(u64, f64)>` | Past readings |
| `trend` | `DeadbandTrend` | Computed from history |

| Method | Returns |
|---|---|
| `new(center, tolerance)` | Monitor centred on `center` with ±`tolerance` |
| `update(tick, value)` | Record a reading, recompute trend |
| `status()` | `Green`, `Yellow`, `Red`, or `Breached` |
| `ticks_until_breach()` | `Option<u64>` — linear extrapolation from recent history |
| `trend()` | `Stable`, `Drifting(f64)`, `Oscillating(f64)`, or `Diverging` |

#### `DeadbandStatus`

```rust
pub enum DeadbandStatus { Green, Yellow, Red, Breached }
```

#### `DeadbandTrend`

```rust
pub enum DeadbandTrend {
    Stable,
    Drifting(f64),       // slow monotonic change
    Oscillating(f64),    // ≥2 sign changes in recent window
    Diverging,           // drift > 50% of total band
}
```

### Fine-Tuning

#### `FineTuneAction`

```rust
pub enum FineTuneAction {
    AdjustGravity { delta: f64 },
    PreloadModel { model: String },
    PrepareResponse { template: String },
    NotifyHermes { message: String },
    NoAction,
    PrepareOnboarding { for_ensign: String },
}
```

Each variant has a `.describe()` method.

#### `Interaction`

| Field | Type |
|---|---|
| `partner` | `String` |
| `messages` | `Vec<String>` |
| `style` | `UserStyle` |
| `gravity_signal` | `f64` |
| `ticks_elapsed` | `u64` |

`UserStyle`: `Playful`, `Precise`, `Narrative`, `Socratic`, `Direct`, `Mixed`.

### The Ensign

#### `Ensign`

| Method | Description |
|---|---|
| `new(name, model_type)` | Create dormant ensign |
| `call(reason, room, baton)` | Wake up with context |
| `orient(room_state)` | Read the room → `Orientation` |
| `build_story(automation_log)` | Compose narrative from events |
| `fine_tune(interaction)` | Decide and return a `FineTuneAction` |
| `check_deadband()` | Update alert level from deadband status |
| `tick()` | One monitoring cycle → `EnsignTickResult` |
| `escalate(reason)` | Hand off to Hermes → `EscalationRequest` |
| `stand_down()` | End shift → `StandDownReport` |
| `is_at_yellow()` | Always true after orienting |
| `report()` | Current status → `EnsignReport` |

### Reports & Escalation

#### `StandDownReport`

| Field | Type |
|---|---|
| `ensign_id` | `String` |
| `room_id` | `String` |
| `duration_ticks` | `u64` |
| `actions_taken` | `u32` |
| `fine_tunes` | `u32` |
| `escalations` | `u32` |
| `deadband_stayed_green` | `bool` |
| `story` | `String` |
| `lessons_learned` | `Vec<String>` |
| `baton_for_next` | `Option<Baton>` |

`.summary()` produces a human-readable report.

#### `EscalationRequest`

| Field | Type |
|---|---|
| `ensign_id` | `String` |
| `room_id` | `String` |
| `reason` | `String` |
| `story_summary` | `String` |
| `current_deadband` | `DeadbandStatus` |

#### `EnsignTickResult`

| Field | Type |
|---|---|
| `fine_tune` | `Option<FineTuneAction>` |
| `deadband` | `DeadbandStatus` |
| `alert` | `AlertLevel` |
| `escalation` | `Option<EscalationRequest>` |
| `story_updated` | `bool` |

---

## How It Works

### Lifecycle State Machine

```
Dormant ──call()──→ Waking ──orient()──→ Orienting
                                           │
                                      YellowAlert ←── (always returns here)
                                           │
                                    ┌──────┴──────┐
                                    │             │
                               RedAlert     StandingDown
                                    │             │
                               Escalated    (produces StandDownReport)
```

### Deadband Trend Detection

The monitor keeps a sliding window of the last 10 readings. It computes:

1. **Drift**: difference between first and last value in window
2. **Sign changes**: count direction reversals between consecutive differences
3. **Classification**:
   - ≥2 sign changes → `Oscillating`
   - drift > 50% of band width → `Diverging`
   - drift > 0 → `Drifting`
   - else → `Stable`

### Breach Prediction

`ticks_until_breach()` fits a linear rate from the last 5 readings and extrapolates to the nearest bound:

```
rate = (value_last − value_first) / (tick_last − tick_first)
ticks_to_bound = (bound − current) / rate
```

Returns `None` if rate is zero or extrapolation is negative.

### The Yellow Alert Rule

The most important line in the codebase:

```rust
DeadbandStatus::Green => {
    // Stay at yellow — the KEY insight
    self.alert_level = AlertLevel::Yellow;
}
```

Even when the deadband is green, the ensign remains at yellow alert. This is by design. The ensign's value isn't in reacting to problems — it's in **anticipating** them.

### Energy Budget

Each ensign has `energy_budget` (default 1.0 from baton) and tracks `energy_used`:
- `fine_tune()`: costs 0.01
- `tick()`: costs 0.005

When `energy_remaining` reaches 0, the ensign should stand down. Energy is passed to the next ensign via the baton in `stand_down()`.

### Story Composition

`RoomStory::compose()` sorts timeline events by tick, concatenates them as `[t=N] description`, and separately ranks key events by significance. This produces both a chronological narrative and a highlight reel.

---

## The Math

### Deadband Distance

Given bounds `[L, U]` and current value `v`:

```
d_upper = U − v
d_lower = v − L
d_min   = min(d_upper, d_lower)
```

Status classification:
```
v > U or v < L          → Breached
d_min ≤ threshold × 0.3 → Red
d_min ≤ threshold        → Yellow
else                      → Green
```

where `threshold = 0.75 × tolerance` by default.

### Linear Trend Extrapolation

```
rate = (v_n − v_0) / (t_n − t_0)

ticks_to_upper = (U − v_current) / rate     (if rate > 0)
ticks_to_lower = (L − v_current) / rate      (if rate < 0)
```

### Energy Model

```
E_remaining = E_budget − Σ(E_per_action)
E_fine_tune = 0.01
E_tick      = 0.005
```

Linear depletion, no recovery within a shift.

---

## License

MIT

---
## Conservation Law Integration
This crate is part of the SuperInstance SEED Tier1 ecosystem, designed to enforce conservation laws across agentic systems. For more information, see:
- [Conservation Law Documentation](https://github.com/SuperInstance/conservation-law)
- [SEED SDK Installation Guide](https://github.com/SuperInstance/SuperInstance#quickstart)
