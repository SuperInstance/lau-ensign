# lau-ensign

The DJ system. Small models that wake up on-call, orient themselves from the baton and room state, and keep the room fine-tuned at **yellow alert even when green would be fine**.

Any decent algorithm gets them dancing. The ensign is constantly readying the *next* transition — reading the room at a sample rate that has blurred from discrete ticks into a continuous dial.

## The concept in 60 seconds

An ensign is a lightweight on-call agent backed by a small model (local or remote). It carries a **baton** (context from the previous ensign), reads the **room state** (tiles, automations, interactions), and produces **fine-tune actions** to keep the room in good shape.

The key insight: ensigns run at **yellow alert by default**. Even when the deadband says "green, everything's fine," the ensign is actively predicting trends and preparing the next adjustment. Deadband trend prediction turns reactive monitoring into proactive tuning.

## Quick start

```rust
use lau_ensign::*;

// Create an ensign backed by a local tiny model
let mut ensign = Ensign::new("bridge-ensign-1", ModelType::LocalTiny)
    .with_call_reason(CallReason::OnDuty);

// Orient from a baton (context handoff from previous ensign)
let baton = Baton::new("previous session notes go here")
    .with_room_state("bridge-active");
ensign.orient(&baton);

// Tick — the ensign reads the room and decides what to do
let room = RoomState::new("bridge")
    .with_active_automations(3)
    .with_pending_interactions(1);
let result = ensign.tick(&room);

// Check what happened
println!("Alert level: {:?}", result.alert_level);
println!("Fine-tune actions: {:?}", result.fine_tune_actions);
println!("Deadband trend: {:?}", result.deadband_trend);
```

## Key types

| Type | What it does |
|------|-------------|
| `Ensign` | The agent itself: model, state, orientation, tick loop |
| `Baton` | Context handoff between ensign rotations |
| `RoomState` | Snapshot of room: automations, interactions, tiles |
| `Orientation` | The ensign's understanding of its room and duties |
| `RoomStory` | Rewound history of automation events in a room |
| `DeadbandMonitor` | Tracks deadband status and predicts trends |
| `AlertLevel` | Green / Yellow / Red / Breached |
| `FineTuneAction` | Specific adjustment the ensign recommends |
| `EnsignReport` | Full status report after a tick |

## Alert levels and deadband

```rust
let monitor = DeadbandMonitor::new(0.1, 1.0);
monitor.update(sensor_value);

let status = monitor.status();     // Green / Yellow / Red / Breached
let trend = monitor.predict_trend(); // Stable / Drifting / Oscillating / Diverging
```

The ensign uses trend prediction to stay at yellow alert — ready to act before the deadband is actually breached.

## Fine-tune actions

```rust
// The ensign produces concrete actions, not just alerts
match action {
    FineTuneAction::AdjustParameter { name, delta } => { /* tweak */ }
    FineTuneAction::EscalateToHuman { reason } => { /* call for help */ }
    FineTuneAction::RewindAutomation { event_id } => { /* undo */ }
    FineTuneAction::StandDown { report } => { /* hand off baton */ }
}
```

## Model types

```rust
ModelType::LocalTiny      // Phi-3-mini, Gemma-2B — fast, local
ModelType::LocalVision    // Moondream — sees images
ModelType::LocalAudio     // Audio processing
ModelType::LocalJEPA      // JEPA predictor
ModelType::RemoteLight    // Seed-mini, GLM-4-flash — small remote
ModelType::RemoteVision   // Small remote with vision
```

## Contributing

PRs welcome. This crate is part of the [SuperInstance](https://github.com/SuperInstance) ecosystem. The ensign model is opinionated — if you want to experiment with different alert strategies or deadband algorithms, open an issue to discuss.
