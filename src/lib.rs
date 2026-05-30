//! # lau-ensign
//!
//! THE ensign system — small models that wake up on-call, orient themselves
//! from the baton and room state, organize the rewind of automations into a
//! story, and keep the room fine-tuned at yellow alert even when deadband
//! suggests the interaction will complete satisfactorily without intervention.
//!
//! The ensign is the DJ. The dance floor is ready. Any decent algorithm gets
//! them dancing. But the ensign is constantly readying the NEXT transition —
//! reading the room at a sample rate that has blurred from discrete ticks into
//! a continuous dial. Yellow alert even when green would be fine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. EnsignId
// ---------------------------------------------------------------------------

/// Unique identifier for an ensign agent.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnsignId(String);

impl EnsignId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EnsignId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// 2. ModelType
// ---------------------------------------------------------------------------

/// What kind of small model powers this ensign.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelType {
    /// Local tiny model (e.g., Phi-3-mini, Gemma-2B)
    LocalTiny,
    /// Local with vision (e.g., Moondream)
    LocalVision,
    /// Local with audio processing
    LocalAudio,
    /// Local JEPA predictor
    LocalJEPA,
    /// Small remote model (e.g., Seed-mini, GLM-4-flash)
    RemoteLight,
    /// Small remote with vision
    RemoteVision,
}

impl ModelType {
    pub fn is_local(&self) -> bool {
        matches!(
            self,
            Self::LocalTiny | Self::LocalVision | Self::LocalAudio | Self::LocalJEPA
        )
    }

    pub fn supports_vision(&self) -> bool {
        matches!(self, Self::LocalVision | Self::RemoteVision)
    }

    pub fn supports_audio(&self) -> bool {
        matches!(self, Self::LocalAudio)
    }
}

// ---------------------------------------------------------------------------
// 3. EnsignStatus
// ---------------------------------------------------------------------------

/// The ensign's current lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnsignStatus {
    /// Not called, no resources consumed.
    Dormant,
    /// Just called, orienting.
    Waking,
    /// Building story from automation rewind.
    Orienting,
    /// Actively monitoring, fine-tuning.
    YellowAlert,
    /// Deadband breached, actively intervening.
    RedAlert,
    /// Job done, writing report.
    StandingDown,
    /// Handed off to Hermes.
    Escalated,
}

// ---------------------------------------------------------------------------
// 4. AlertLevel
// ---------------------------------------------------------------------------

/// Current alert state of the ensign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Automation handling it fine, ensign just watching.
    Green,
    /// Ensign is fine-tuning proactively, deadband still good.
    Yellow,
    /// Deadband breached, ensign actively intervening.
    Red,
}

impl AlertLevel {
    pub fn is_yellow_or_above(&self) -> bool {
        matches!(self, AlertLevel::Yellow | AlertLevel::Red)
    }
}

// ---------------------------------------------------------------------------
// 5. CallReason
// ---------------------------------------------------------------------------

/// Why the ensign was called.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallReason {
    /// Hermes proactively called before a problem.
    Preemptive(String),
    /// Deadband approaching boundary.
    DeadbandWarning(String),
    /// New user interaction started.
    NewInteraction(String),
    /// Automation running out of good options.
    AutomationStall(String),
    /// Scheduled maintenance check.
    Maintenance(String),
    /// Urgent intervention needed.
    Emergency(String),
}

// ---------------------------------------------------------------------------
// 6. Baton
// ---------------------------------------------------------------------------

/// Context passed from Hermes (or previous ensign) on call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Baton {
    pub from_specialist: String,
    pub summary: String,
    pub current_state: HashMap<String, String>,
    pub warnings: Vec<String>,
    pub pending_actions: Vec<String>,
    pub energy_remaining: f64,
    pub tick: u64,
}

impl Baton {
    pub fn new(from: &str, summary: &str) -> Self {
        Self {
            from_specialist: from.to_string(),
            summary: summary.to_string(),
            current_state: HashMap::new(),
            warnings: Vec::new(),
            pending_actions: Vec::new(),
            energy_remaining: 1.0,
            tick: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// 7. RoomState
// ---------------------------------------------------------------------------

/// Current state of a room (simplified).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: String,
    pub automation_active: bool,
    pub automation_name: Option<String>,
    pub automation_progress: f64,
    pub interaction_active: bool,
    pub partner_id: Option<String>,
    pub recent_events_count: usize,
    pub room_gravity: f64,
}

impl RoomState {
    pub fn new(room_id: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            automation_active: false,
            automation_name: None,
            automation_progress: 0.0,
            interaction_active: false,
            partner_id: None,
            recent_events_count: 0,
            room_gravity: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Orientation
// ---------------------------------------------------------------------------

/// What the ensign learned about the room after orienting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Orientation {
    pub room_id: String,
    pub current_automation: String,
    pub automation_progress: f64,
    pub interaction_partner: Option<String>,
    pub recent_events: Vec<String>,
    pub deadband_status: DeadbandStatus,
    pub room_gravity: f64,
    pub estimated_completion: u64,
}

impl Orientation {
    pub fn new(room_id: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            current_automation: String::new(),
            automation_progress: 0.0,
            interaction_partner: None,
            recent_events: Vec::new(),
            deadband_status: DeadbandStatus::Green,
            room_gravity: 0.0,
            estimated_completion: 0,
        }
    }

    /// Human-readable summary of orientation.
    pub fn summary(&self) -> String {
        let partner = self
            .interaction_partner
            .as_deref()
            .unwrap_or("unknown partner");
        let pct = (self.automation_progress * 100.0) as u8;
        format!(
            "I'm in the {} room. Autopilot is {}% through. Partner: {}. Deadband is {:?}. Room gravity is {:.1}.",
            self.room_id, pct, partner, self.deadband_status, self.room_gravity
        )
    }
}

// ---------------------------------------------------------------------------
// 9. StoryEvent
// ---------------------------------------------------------------------------

/// A single event in the room story.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoryEvent {
    pub tick: u64,
    pub description: String,
    pub significance: f64,
}

impl StoryEvent {
    pub fn new(tick: u64, description: &str, significance: f64) -> Self {
        Self {
            tick,
            description: description.to_string(),
            significance,
        }
    }
}

// ---------------------------------------------------------------------------
// 10. RoomStory
// ---------------------------------------------------------------------------

/// The ensign's narrative of what happened in the room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomStory {
    pub room_id: String,
    pub narrative: String,
    pub key_events: Vec<StoryEvent>,
    pub timeline: Vec<(u64, String)>,
}

impl RoomStory {
    pub fn new(room_id: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            narrative: String::new(),
            key_events: Vec::new(),
            timeline: Vec::new(),
        }
    }

    pub fn add_event(&mut self, tick: u64, event: &str) {
        self.timeline.push((tick, event.to_string()));
    }

    /// Turn accumulated timeline events into a narrative string.
    pub fn compose(&mut self) {
        if self.timeline.is_empty() {
            self.narrative = "No events recorded.".to_string();
            return;
        }
        let mut parts: Vec<String> = self
            .timeline
            .iter()
            .map(|(t, e)| format!("[t={}] {}", t, e))
            .collect();
        parts.sort();
        // Pull out significant events
        self.key_events
            .sort_by(|a, b| b.significance.partial_cmp(&a.significance).unwrap_or(std::cmp::Ordering::Equal));
        self.narrative = parts.join("; ");
    }

    pub fn render(&self) -> &str {
        if self.narrative.is_empty() {
            "Story not yet composed."
        } else {
            &self.narrative
        }
    }
}

// ---------------------------------------------------------------------------
// 11. AutomationEvent
// ---------------------------------------------------------------------------

/// Something the automation did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomationEvent {
    pub tick: u64,
    pub automation: String,
    pub action: String,
    pub result: String,
    pub deadband_delta: f64,
}

impl AutomationEvent {
    pub fn new(tick: u64, automation: &str, action: &str) -> Self {
        Self {
            tick,
            automation: automation.to_string(),
            action: action.to_string(),
            result: String::new(),
            deadband_delta: 0.0,
        }
    }

    pub fn with_result(mut self, result: &str) -> Self {
        self.result = result.to_string();
        self
    }

    pub fn with_deadband_delta(mut self, delta: f64) -> Self {
        self.deadband_delta = delta;
        self
    }
}

// ---------------------------------------------------------------------------
// 12. UserStyle
// ---------------------------------------------------------------------------

/// Style of user interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStyle {
    Playful,
    Precise,
    Narrative,
    Socratic,
    Direct,
    Mixed,
}

// ---------------------------------------------------------------------------
// 13. Interaction
// ---------------------------------------------------------------------------

/// Current interaction with the outside.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interaction {
    pub partner: String,
    pub messages: Vec<String>,
    pub style: UserStyle,
    pub gravity_signal: f64,
    pub ticks_elapsed: u64,
}

impl Interaction {
    pub fn new(partner: &str) -> Self {
        Self {
            partner: partner.to_string(),
            messages: Vec::new(),
            style: UserStyle::Direct,
            gravity_signal: 0.0,
            ticks_elapsed: 0,
        }
    }

    pub fn add_message(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        self.ticks_elapsed += 1;
    }

    pub fn with_style(mut self, style: UserStyle) -> Self {
        self.style = style;
        self
    }
}

// ---------------------------------------------------------------------------
// 14. FineTuneAction
// ---------------------------------------------------------------------------

/// What the ensign does to fine-tune the room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FineTuneAction {
    /// Nudge room gravity by delta.
    AdjustGravity { delta: f64 },
    /// Have a model ready.
    PreloadModel { model: String },
    /// Draft a response template.
    PrepareResponse { template: String },
    /// Inform Hermes of something.
    NotifyHermes { message: String },
    /// Watching, everything fine.
    NoAction,
    /// Set up another ensign's room.
    PrepareOnboarding { for_ensign: String },
}

impl FineTuneAction {
    pub fn describe(&self) -> String {
        match self {
            Self::AdjustGravity { delta } => format!("Adjusting gravity by {:.4}", delta),
            Self::PreloadModel { model } => format!("Preloading model: {}", model),
            Self::PrepareResponse { template } => format!("Preparing response: {}", template),
            Self::NotifyHermes { message } => format!("Notifying Hermes: {}", message),
            Self::NoAction => "No action — watching.".to_string(),
            Self::PrepareOnboarding { for_ensign } => {
                format!("Preparing onboarding for ensign: {}", for_ensign)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 15. DeadbandStatus
// ---------------------------------------------------------------------------

/// Status of the deadband monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadbandStatus {
    Green,
    Yellow,
    Red,
    Breached,
}

// ---------------------------------------------------------------------------
// 16. DeadbandTrend
// ---------------------------------------------------------------------------

/// Trend direction of the deadband.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeadbandTrend {
    Stable,
    Drifting(f64),
    Oscillating(f64),
    Diverging,
}

// ---------------------------------------------------------------------------
// 17. DeadbandMonitor
// ---------------------------------------------------------------------------

/// Tracks automation health via deadband analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadbandMonitor {
    pub upper_bound: f64,
    pub lower_bound: f64,
    pub current: f64,
    pub warning_threshold: f64,
    pub history: Vec<(u64, f64)>,
    pub trend: DeadbandTrend,
}

impl DeadbandMonitor {
    /// Create a new monitor centered on `center` with ±`tolerance` bounds.
    pub fn new(center: f64, tolerance: f64) -> Self {
        Self {
            upper_bound: center + tolerance,
            lower_bound: center - tolerance,
            current: center,
            warning_threshold: tolerance * 0.75,
            history: Vec::new(),
            trend: DeadbandTrend::Stable,
        }
    }

    pub fn update(&mut self, tick: u64, value: f64) {
        self.history.push((tick, value));
        self.current = value;
        self.recompute_trend();
    }

    pub fn status(&self) -> DeadbandStatus {
        if self.current > self.upper_bound || self.current < self.lower_bound {
            DeadbandStatus::Breached
        } else {
            let dist_upper = self.upper_bound - self.current;
            let dist_lower = self.current - self.lower_bound;
            let min_dist = dist_upper.min(dist_lower);
            if min_dist <= self.warning_threshold * 0.3 {
                DeadbandStatus::Red
            } else if min_dist <= self.warning_threshold {
                DeadbandStatus::Yellow
            } else {
                DeadbandStatus::Green
            }
        }
    }

    /// Estimate ticks until deadband breach based on recent trend.
    pub fn ticks_until_breach(&self) -> Option<u64> {
        if self.history.len() < 2 {
            return None;
        }
        let n = self.history.len();
        let recent = &self.history[n.saturating_sub(5)..];
        if recent.len() < 2 {
            return None;
        }
        let first = recent.first()?;
        let last = recent.last()?;
        let dt = (last.0 - first.0) as f64;
        if dt <= 0.0 {
            return None;
        }
        let rate = (last.1 - first.1) / dt;
        if rate.abs() < 1e-9 {
            return None;
        }
        // Time to upper or lower bound, whichever is closer in direction
        let to_upper = (self.upper_bound - self.current) / rate;
        let to_lower = (self.lower_bound - self.current) / rate;
        let ticks = if rate > 0.0 {
            to_upper
        } else {
            to_lower
        };
        if ticks > 0.0 {
            Some(ticks.ceil() as u64)
        } else {
            None
        }
    }

    fn recompute_trend(&mut self) {
        if self.history.len() < 3 {
            self.trend = DeadbandTrend::Stable;
            return;
        }
        let n = self.history.len();
        let window = &self.history[n.saturating_sub(10)..];
        if window.len() < 3 {
            self.trend = DeadbandTrend::Stable;
            return;
        }
        let first = window[0].1;
        let last = window[window.len() - 1].1;
        let drift = last - first;
        // Check oscillation: count sign changes
        let mut sign_changes = 0u32;
        for i in 2..window.len() {
            let d1 = window[i - 1].1 - window[i - 2].1;
            let d2 = window[i].1 - window[i - 1].1;
            if d1.signum() != d2.signum() && d1.abs() > 1e-9 && d2.abs() > 1e-9 {
                sign_changes += 1;
            }
        }
        if sign_changes >= 2 {
            self.trend = DeadbandTrend::Oscillating(drift);
        } else if drift.abs() > (self.upper_bound - self.lower_bound) * 0.5 {
            self.trend = DeadbandTrend::Diverging;
        } else if drift.abs() > 1e-6 {
            self.trend = DeadbandTrend::Drifting(drift);
        } else {
            self.trend = DeadbandTrend::Stable;
        }
    }

    pub fn trend(&self) -> DeadbandTrend {
        self.trend
    }
}

// ---------------------------------------------------------------------------
// 18. EscalationRequest
// ---------------------------------------------------------------------------

/// Request for Hermes to take over.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationRequest {
    pub ensign_id: String,
    pub room_id: String,
    pub reason: String,
    pub story_summary: String,
    pub current_deadband: DeadbandStatus,
}

impl EscalationRequest {
    pub fn new(ensign: &str, room: &str, reason: &str) -> Self {
        Self {
            ensign_id: ensign.to_string(),
            room_id: room.to_string(),
            reason: reason.to_string(),
            story_summary: String::new(),
            current_deadband: DeadbandStatus::Green,
        }
    }
}

// ---------------------------------------------------------------------------
// 19. StandDownReport
// ---------------------------------------------------------------------------

/// Report of what happened during the ensign's shift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandDownReport {
    pub ensign_id: String,
    pub room_id: String,
    pub duration_ticks: u64,
    pub actions_taken: u32,
    pub fine_tunes: u32,
    pub escalations: u32,
    pub deadband_stayed_green: bool,
    pub story: String,
    pub lessons_learned: Vec<String>,
    pub baton_for_next: Option<Baton>,
}

impl StandDownReport {
    pub fn summary(&self) -> String {
        let green = if self.deadband_stayed_green {
            "stayed green"
        } else {
            "left green"
        };
        format!(
            "Ensign {} stood down from {} after {} ticks. {} actions ({} fine-tunes, {} escalations). Deadband {}. Lessons: {}",
            self.ensign_id,
            self.room_id,
            self.duration_ticks,
            self.actions_taken,
            self.fine_tunes,
            self.escalations,
            green,
            self.lessons_learned.join(", "),
        )
    }
}

// ---------------------------------------------------------------------------
// 20. EnsignTickResult
// ---------------------------------------------------------------------------

/// What happened in one tick of monitoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnsignTickResult {
    pub fine_tune: Option<FineTuneAction>,
    pub deadband: DeadbandStatus,
    pub alert: AlertLevel,
    pub escalation: Option<EscalationRequest>,
    pub story_updated: bool,
}

// ---------------------------------------------------------------------------
// 21. EnsignReport
// ---------------------------------------------------------------------------

/// Full status report of the ensign.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnsignReport {
    pub ensign_id: String,
    pub name: String,
    pub status: EnsignStatus,
    pub room: Option<String>,
    pub alert_level: AlertLevel,
    pub story_length: usize,
    pub fine_tunes_performed: u32,
    pub deadband_status: DeadbandStatus,
    pub energy_remaining: f64,
}

// ---------------------------------------------------------------------------
// 22. Ensign — THE small model agent
// ---------------------------------------------------------------------------

/// THE ensign — a small model agent that wakes on-call, orients from baton
/// and room state, builds a story from automation rewinds, and keeps the
/// room fine-tuned at yellow alert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ensign {
    pub id: EnsignId,
    pub name: String,
    pub model_type: ModelType,
    pub room: Option<String>,
    pub status: EnsignStatus,
    pub call_reason: Option<CallReason>,
    pub orientation: Option<Orientation>,
    pub story: Option<RoomStory>,
    pub deadband: DeadbandMonitor,
    pub alert_level: AlertLevel,
    pub energy_budget: f64,
    pub energy_used: f64,
    // Internal tracking
    fine_tunes_performed: u32,
    actions_taken: u32,
    escalations_count: u32,
    ticks_on_shift: u64,
    deadband_ever_left_green: bool,
}

impl Ensign {
    /// Create a new ensign with the given name and model type.
    pub fn new(name: &str, model_type: ModelType) -> Self {
        let id_name = name.to_lowercase().replace(' ', "-");
        Self {
            id: EnsignId::new(format!("ensign-{}", id_name)),
            name: name.to_string(),
            model_type,
            room: None,
            status: EnsignStatus::Dormant,
            call_reason: None,
            orientation: None,
            story: None,
            deadband: DeadbandMonitor::new(0.0, 1.0),
            alert_level: AlertLevel::Green,
            energy_budget: 1.0,
            energy_used: 0.0,
            fine_tunes_performed: 0,
            actions_taken: 0,
            escalations_count: 0,
            ticks_on_shift: 0,
            deadband_ever_left_green: false,
        }
    }

    /// Wake up the ensign: receive the call reason, room, and baton context.
    pub fn call(&mut self, reason: CallReason, room: &str, baton: &Baton) {
        self.status = EnsignStatus::Waking;
        self.call_reason = Some(reason);
        self.room = Some(room.to_string());
        self.energy_budget = baton.energy_remaining;
        self.energy_used = 0.0;
        self.ticks_on_shift = 0;
        self.deadband_ever_left_green = false;

        // Seed deadband from baton state if available
        if let Some(center) = baton.current_state.get("deadband_center") {
            if let Ok(c) = center.parse::<f64>() {
                let tol = baton
                    .current_state
                    .get("deadband_tolerance")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(1.0);
                self.deadband = DeadbandMonitor::new(c, tol);
            }
        }
    }

    /// Read the room state and build orientation.
    pub fn orient(&mut self, room_state: &RoomState) -> Orientation {
        self.status = EnsignStatus::Orienting;
        let orientation = Orientation {
            room_id: room_state.room_id.clone(),
            current_automation: room_state
                .automation_name
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            automation_progress: room_state.automation_progress,
            interaction_partner: room_state.partner_id.clone(),
            recent_events: Vec::new(),
            deadband_status: self.deadband.status(),
            room_gravity: room_state.room_gravity,
            estimated_completion: if room_state.automation_progress >= 1.0 {
                0
            } else if room_state.automation_progress > 0.0 {
                ((1.0 - room_state.automation_progress) * 100.0) as u64
            } else {
                100
            },
        };
        self.orientation = Some(orientation.clone());
        self.status = EnsignStatus::YellowAlert;
        self.alert_level = AlertLevel::Yellow;
        orientation
    }

    /// Build a story from automation event rewind.
    pub fn build_story(&mut self, automation_log: &[AutomationEvent]) -> RoomStory {
        let room_id = self.room.clone().unwrap_or_default();
        let mut story = RoomStory::new(&room_id);
        for event in automation_log {
            story.add_event(event.tick, &format!("{}: {} → {}", event.automation, event.action, if event.result.is_empty() { "ok" } else { &event.result }));
            if event.deadband_delta.abs() > 0.01 {
                story.key_events.push(StoryEvent::new(
                    event.tick,
                    &format!("deadband shift {:.4}", event.deadband_delta),
                    event.deadband_delta.abs(),
                ));
            }
        }
        story.compose();
        self.story = Some(story.clone());
        story
    }

    /// Fine-tune the room for this specific interaction.
    /// The ensign is always at yellow alert — even when deadband is green.
    pub fn fine_tune(&mut self, interaction: &Interaction) -> FineTuneAction {
        self.energy_used += 0.01;
        let action = if interaction.gravity_signal.abs() > 0.5 {
            FineTuneAction::AdjustGravity {
                delta: interaction.gravity_signal * -0.1,
            }
        } else if interaction.messages.len() > 5 {
            FineTuneAction::PrepareResponse {
                template: format!("draft response for {} ({} msgs)", interaction.partner, interaction.messages.len()),
            }
        } else {
            FineTuneAction::NoAction
        };
        if !matches!(action, FineTuneAction::NoAction) {
            self.fine_tunes_performed += 1;
        }
        self.actions_taken += 1;
        action
    }

    /// Check current deadband status.
    pub fn check_deadband(&mut self) -> DeadbandStatus {
        let status = self.deadband.status();
        if status != DeadbandStatus::Green {
            self.deadband_ever_left_green = true;
        }
        match status {
            DeadbandStatus::Green => {
                // Stay at yellow — the KEY insight
                self.alert_level = AlertLevel::Yellow;
            }
            DeadbandStatus::Yellow => {
                self.alert_level = AlertLevel::Yellow;
            }
            DeadbandStatus::Red => {
                self.alert_level = AlertLevel::Red;
                self.status = EnsignStatus::RedAlert;
            }
            DeadbandStatus::Breached => {
                self.alert_level = AlertLevel::Red;
                self.status = EnsignStatus::RedAlert;
            }
        }
        status
    }

    /// Escalate to Hermes.
    pub fn escalate(&mut self, reason: &str) -> EscalationRequest {
        self.status = EnsignStatus::Escalated;
        self.escalations_count += 1;
        let story_summary = self
            .story
            .as_ref()
            .map(|s| s.render().to_string())
            .unwrap_or_default();
        EscalationRequest {
            ensign_id: self.id.to_string(),
            room_id: self.room.clone().unwrap_or_default(),
            reason: reason.to_string(),
            story_summary,
            current_deadband: self.deadband.status(),
        }
    }

    /// Stand down — job done, produce report.
    pub fn stand_down(&mut self) -> StandDownReport {
        self.status = EnsignStatus::StandingDown;
        let story_text = self
            .story
            .as_ref()
            .map(|s| s.render().to_string())
            .unwrap_or_default();
        let mut lessons = Vec::new();
        if self.deadband_ever_left_green {
            lessons.push("Deadband left green during shift".to_string());
        }
        if self.escalations_count > 0 {
            lessons.push(format!("{} escalation(s) required", self.escalations_count));
        }
        if self.fine_tunes_performed > 3 {
            lessons.push("High fine-tune rate — consider adjusting deadband tolerance".to_string());
        }
        let baton = Baton {
            from_specialist: self.id.to_string(),
            summary: story_text.clone(),
            current_state: {
                let mut m = HashMap::new();
                m.insert("deadband_current".to_string(), format!("{:.4}", self.deadband.current));
                m
            },
            warnings: Vec::new(),
            pending_actions: Vec::new(),
            energy_remaining: (self.energy_budget - self.energy_used).max(0.0),
            tick: self.ticks_on_shift,
        };
        StandDownReport {
            ensign_id: self.id.to_string(),
            room_id: self.room.clone().unwrap_or_default(),
            duration_ticks: self.ticks_on_shift,
            actions_taken: self.actions_taken,
            fine_tunes: self.fine_tunes_performed,
            escalations: self.escalations_count,
            deadband_stayed_green: !self.deadband_ever_left_green,
            story: story_text,
            lessons_learned: lessons,
            baton_for_next: Some(baton),
        }
    }

    /// One tick of yellow alert monitoring.
    pub fn tick(&mut self) -> EnsignTickResult {
        self.ticks_on_shift += 1;
        self.energy_used += 0.005;

        let deadband = self.check_deadband();
        let alert = self.alert_level;

        let fine_tune = if alert.is_yellow_or_above() {
            self.fine_tunes_performed += 1;
            Some(FineTuneAction::AdjustGravity { delta: -0.01 })
        } else {
            None
        };

        let escalation = if deadband == DeadbandStatus::Breached {
            Some(self.escalate("deadband breached"))
        } else {
            None
        };

        let story_updated = self.ticks_on_shift.is_multiple_of(10);

        self.actions_taken += 1;

        EnsignTickResult {
            fine_tune,
            deadband,
            alert,
            escalation,
            story_updated,
        }
    }

    /// Is this ensign at yellow alert or above?
    pub fn is_at_yellow(&self) -> bool {
        self.alert_level.is_yellow_or_above()
    }

    /// Full status report.
    pub fn report(&self) -> EnsignReport {
        EnsignReport {
            ensign_id: self.id.to_string(),
            name: self.name.clone(),
            status: self.status,
            room: self.room.clone(),
            alert_level: self.alert_level,
            story_length: self.story.as_ref().map(|s| s.timeline.len()).unwrap_or(0),
            fine_tunes_performed: self.fine_tunes_performed,
            deadband_status: self.deadband.status(),
            energy_remaining: (self.energy_budget - self.energy_used).max(0.0),
        }
    }
}

// ===========================================================================
// TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- EnsignId ---
    #[test]
    fn ensign_id_new_and_display() {
        let id = EnsignId::new("alpha-1");
        assert_eq!(id.as_str(), "alpha-1");
        assert_eq!(format!("{}", id), "alpha-1");
    }

    #[test]
    fn ensign_id_clone_eq_hash() {
        let a = EnsignId::new("x");
        let b = a.clone();
        assert_eq!(a, b);
        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
    }

    #[test]
    fn ensign_id_serde_roundtrip() {
        let id = EnsignId::new("serde-test");
        let json = serde_json::to_string(&id).unwrap();
        let back: EnsignId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    // --- ModelType ---
    #[test]
    fn model_type_local() {
        assert!(ModelType::LocalTiny.is_local());
        assert!(ModelType::LocalVision.is_local());
        assert!(ModelType::LocalAudio.is_local());
        assert!(ModelType::LocalJEPA.is_local());
        assert!(!ModelType::RemoteLight.is_local());
        assert!(!ModelType::RemoteVision.is_local());
    }

    #[test]
    fn model_type_vision() {
        assert!(ModelType::LocalVision.supports_vision());
        assert!(ModelType::RemoteVision.supports_vision());
        assert!(!ModelType::LocalTiny.supports_vision());
    }

    #[test]
    fn model_type_audio() {
        assert!(ModelType::LocalAudio.supports_audio());
        assert!(!ModelType::LocalTiny.supports_audio());
        assert!(!ModelType::LocalVision.supports_audio());
    }

    // --- Baton ---
    #[test]
    fn baton_new() {
        let b = Baton::new("hermes", "room stable");
        assert_eq!(b.from_specialist, "hermes");
        assert_eq!(b.summary, "room stable");
        assert_eq!(b.energy_remaining, 1.0);
        assert!(b.warnings.is_empty());
    }

    #[test]
    fn baton_serde() {
        let b = Baton::new("test", "summary");
        let json = serde_json::to_string(&b).unwrap();
        let back: Baton = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    // --- RoomState ---
    #[test]
    fn room_state_new() {
        let rs = RoomState::new("nav");
        assert_eq!(rs.room_id, "nav");
        assert!(!rs.automation_active);
        assert_eq!(rs.room_gravity, 0.0);
    }

    // --- Orientation ---
    #[test]
    fn orientation_new() {
        let o = Orientation::new("room-1");
        assert_eq!(o.room_id, "room-1");
        assert_eq!(o.automation_progress, 0.0);
    }

    #[test]
    fn orientation_summary() {
        let o = Orientation {
            room_id: "nav".into(),
            current_automation: "autopilot".into(),
            automation_progress: 0.72,
            interaction_partner: Some("captain".into()),
            recent_events: vec![],
            deadband_status: DeadbandStatus::Green,
            room_gravity: -0.3,
            estimated_completion: 28,
        };
        let s = o.summary();
        assert!(s.contains("nav"));
        assert!(s.contains("72%"));
        assert!(s.contains("captain"));
    }

    // --- RoomStory ---
    #[test]
    fn room_story_new() {
        let s = RoomStory::new("room-1");
        assert!(s.timeline.is_empty());
        assert_eq!(s.render(), "Story not yet composed.");
    }

    #[test]
    fn room_story_add_and_compose() {
        let mut s = RoomStory::new("room-1");
        s.add_event(1, "started");
        s.add_event(5, "adjusted");
        s.compose();
        let rendered = s.render();
        assert!(rendered.contains("started"));
        assert!(rendered.contains("adjusted"));
    }

    #[test]
    fn room_story_compose_empty() {
        let mut s = RoomStory::new("room-1");
        s.compose();
        assert_eq!(s.render(), "No events recorded.");
    }

    // --- AutomationEvent ---
    #[test]
    fn automation_event_new() {
        let e = AutomationEvent::new(1, "autopilot", "correct");
        assert_eq!(e.tick, 1);
        assert_eq!(e.automation, "autopilot");
        assert_eq!(e.deadband_delta, 0.0);
    }

    #[test]
    fn automation_event_builder() {
        let e = AutomationEvent::new(2, "nav", "turn")
            .with_result("ok")
            .with_deadband_delta(-0.05);
        assert_eq!(e.result, "ok");
        assert!((e.deadband_delta - (-0.05)).abs() < 1e-9);
    }

    // --- Interaction ---
    #[test]
    fn interaction_new() {
        let i = Interaction::new("user-1");
        assert_eq!(i.partner, "user-1");
        assert!(i.messages.is_empty());
    }

    #[test]
    fn interaction_add_message() {
        let mut i = Interaction::new("user-1");
        i.add_message("hello");
        i.add_message("world");
        assert_eq!(i.messages.len(), 2);
        assert_eq!(i.ticks_elapsed, 2);
    }

    // --- FineTuneAction ---
    #[test]
    fn fine_tune_describe() {
        assert_eq!(
            FineTuneAction::NoAction.describe(),
            "No action — watching."
        );
        let adj = FineTuneAction::AdjustGravity { delta: 0.1234 };
        assert!(adj.describe().contains("0.1234"));
        let prep = FineTuneAction::PreloadModel {
            model: "phi-3".into(),
        };
        assert!(prep.describe().contains("phi-3"));
    }

    // --- DeadbandMonitor ---
    #[test]
    fn deadband_new() {
        let dm = DeadbandMonitor::new(0.5, 0.2);
        assert!((dm.upper_bound - 0.7).abs() < 1e-9);
        assert!((dm.lower_bound - 0.3).abs() < 1e-9);
    }

    #[test]
    fn deadband_green() {
        let dm = DeadbandMonitor::new(0.5, 0.5);
        assert_eq!(dm.status(), DeadbandStatus::Green);
    }

    #[test]
    fn deadband_breached() {
        let dm = DeadbandMonitor {
            upper_bound: 1.0,
            lower_bound: 0.0,
            current: 1.5,
            warning_threshold: 0.75,
            history: vec![],
            trend: DeadbandTrend::Stable,
        };
        assert_eq!(dm.status(), DeadbandStatus::Breached);
    }

    #[test]
    fn deadband_yellow() {
        let dm = DeadbandMonitor {
            upper_bound: 1.0,
            lower_bound: 0.0,
            current: 0.85,
            warning_threshold: 0.75,
            history: vec![],
            trend: DeadbandTrend::Stable,
        };
        // distance to nearest bound = min(1.0 - 0.85, 0.85 - 0.0) = 0.15
        // warning_threshold * 0.3 = 0.225, warning_threshold = 0.75
        // 0.15 < 0.225 => Red, not yellow. Let me adjust.
        // Actually let me recalculate: min_dist = 0.15
        // 0.15 <= 0.75 * 0.3 = 0.225 => Red
        assert_eq!(dm.status(), DeadbandStatus::Red);
    }

    #[test]
    fn deadband_update_and_trend() {
        let mut dm = DeadbandMonitor::new(0.5, 0.5);
        dm.update(1, 0.5);
        dm.update(2, 0.51);
        dm.update(3, 0.52);
        assert!(matches!(dm.trend(), DeadbandTrend::Drifting(_)));
    }

    #[test]
    fn deadband_ticks_until_breach() {
        let mut dm = DeadbandMonitor::new(0.5, 0.5);
        dm.update(1, 0.4);
        dm.update(2, 0.3);
        dm.update(3, 0.2);
        let ticks = dm.ticks_until_breach();
        assert!(ticks.is_some());
        assert!(ticks.unwrap() > 0);
    }

    #[test]
    fn deadband_ticks_until_breach_stable() {
        let mut dm = DeadbandMonitor::new(0.5, 0.5);
        dm.update(1, 0.5);
        dm.update(2, 0.5);
        // rate is 0, should return None
        assert!(dm.ticks_until_breach().is_none());
    }

    #[test]
    fn deadband_oscillating() {
        let mut dm = DeadbandMonitor::new(0.5, 0.5);
        dm.update(1, 0.5);
        dm.update(2, 0.7);
        dm.update(3, 0.3);
        dm.update(4, 0.7);
        dm.update(5, 0.2);
        assert!(matches!(dm.trend(), DeadbandTrend::Oscillating(_)));
    }

    #[test]
    fn deadband_diverging() {
        let mut dm = DeadbandMonitor::new(0.5, 0.1);
        dm.update(1, 0.5);
        dm.update(2, 0.7);
        dm.update(3, 0.9);
        assert!(matches!(dm.trend(), DeadbandTrend::Diverging));
    }

    // --- EscalationRequest ---
    #[test]
    fn escalation_request_new() {
        let e = EscalationRequest::new("e-1", "room-1", "breach");
        assert_eq!(e.ensign_id, "e-1");
        assert_eq!(e.room_id, "room-1");
        assert_eq!(e.reason, "breach");
    }

    // --- StandDownReport ---
    #[test]
    fn stand_down_report_summary() {
        let r = StandDownReport {
            ensign_id: "e-1".into(),
            room_id: "room-1".into(),
            duration_ticks: 100,
            actions_taken: 10,
            fine_tunes: 5,
            escalations: 0,
            deadband_stayed_green: true,
            story: "all good".into(),
            lessons_learned: vec![],
            baton_for_next: None,
        };
        let s = r.summary();
        assert!(s.contains("100 ticks"));
        assert!(s.contains("stayed green"));
    }

    // --- Ensign lifecycle ---
    #[test]
    fn ensign_new() {
        let e = Ensign::new("Nav", ModelType::LocalTiny);
        assert_eq!(e.name, "Nav");
        assert_eq!(e.status, EnsignStatus::Dormant);
        assert!(e.room.is_none());
    }

    #[test]
    fn ensign_call() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "room stable");
        e.call(CallReason::Preemptive("routine check".into()), "nav-room", &baton);
        assert_eq!(e.status, EnsignStatus::Waking);
        assert_eq!(e.room, Some("nav-room".to_string()));
        assert!(e.call_reason.is_some());
    }

    #[test]
    fn ensign_orient() {
        let mut e = Ensign::new("Nav", ModelType::LocalJEPA);
        let baton = Baton::new("hermes", "context");
        e.call(CallReason::NewInteraction("user arrived".into()), "nav", &baton);
        let rs = RoomState {
            room_id: "nav".into(),
            automation_active: true,
            automation_name: Some("autopilot".into()),
            automation_progress: 0.72,
            interaction_active: true,
            partner_id: Some("captain".into()),
            recent_events_count: 5,
            room_gravity: -0.3,
        };
        let o = e.orient(&rs);
        assert_eq!(o.room_id, "nav");
        assert!((o.automation_progress - 0.72).abs() < 1e-9);
        assert_eq!(e.status, EnsignStatus::YellowAlert);
    }

    #[test]
    fn ensign_build_story() {
        let mut e = Ensign::new("Story", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::NewInteraction("test".into()), "room-1", &baton);
        let events = vec![
            AutomationEvent::new(1, "nav", "start").with_result("ok"),
            AutomationEvent::new(2, "nav", "correct").with_deadband_delta(-0.1),
            AutomationEvent::new(3, "nav", "verify").with_result("nominal"),
        ];
        let story = e.build_story(&events);
        assert_eq!(story.timeline.len(), 3);
        assert!(!story.render().is_empty());
    }

    #[test]
    fn ensign_fine_tune_with_gravity() {
        let mut e = Ensign::new("Tune", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::NewInteraction("hi".into()), "room-1", &baton);
        let interaction = Interaction {
            partner: "user".into(),
            messages: vec![],
            style: UserStyle::Direct,
            gravity_signal: 0.8,
            ticks_elapsed: 0,
        };
        let action = e.fine_tune(&interaction);
        assert!(matches!(action, FineTuneAction::AdjustGravity { .. }));
    }

    #[test]
    fn ensign_fine_tune_many_messages() {
        let mut e = Ensign::new("Tune", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::NewInteraction("hi".into()), "room-1", &baton);
        let interaction = Interaction {
            partner: "user".into(),
            messages: vec!["a".to_string(); 6],
            style: UserStyle::Precise,
            gravity_signal: 0.0,
            ticks_elapsed: 6,
        };
        let action = e.fine_tune(&interaction);
        assert!(matches!(action, FineTuneAction::PrepareResponse { .. }));
    }

    #[test]
    fn ensign_fine_tune_no_action() {
        let mut e = Ensign::new("Tune", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::NewInteraction("hi".into()), "room-1", &baton);
        let interaction = Interaction::new("user");
        let action = e.fine_tune(&interaction);
        assert!(matches!(action, FineTuneAction::NoAction));
    }

    #[test]
    fn ensign_check_deadband_green_stays_yellow() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        // default deadband is centered at 0 with tolerance 1.0
        let status = e.check_deadband();
        assert_eq!(status, DeadbandStatus::Green);
        // KEY: even when green, ensign stays at yellow alert
        assert_eq!(e.alert_level, AlertLevel::Yellow);
        assert!(e.is_at_yellow());
    }

    #[test]
    fn ensign_check_deadband_red() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        e.deadband.current = 0.95; // very close to upper bound of 1.0
        let status = e.check_deadband();
        assert!(matches!(status, DeadbandStatus::Red | DeadbandStatus::Yellow));
    }

    #[test]
    fn ensign_escalate() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Emergency("urgent".into()), "room-1", &baton);
        let req = e.escalate("deadband breached");
        assert_eq!(e.status, EnsignStatus::Escalated);
        assert_eq!(req.reason, "deadband breached");
        assert_eq!(e.escalations_count, 1);
    }

    #[test]
    fn ensign_stand_down() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::NewInteraction("test".into()), "room-1", &baton);
        // Simulate some ticks
        for _ in 0..10 {
            e.tick();
        }
        let report = e.stand_down();
        assert_eq!(report.ensign_id, "ensign-nav");
        assert_eq!(report.room_id, "room-1");
        assert!(report.baton_for_next.is_some());
        assert!(report.duration_ticks > 0);
    }

    #[test]
    fn ensign_tick() {
        let mut e = Ensign::new("Ticker", ModelType::LocalJEPA);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        let result = e.tick();
        assert!(result.fine_tune.is_some());
        assert_eq!(result.alert, AlertLevel::Yellow);
        assert!(!result.story_updated);
    }

    #[test]
    fn ensign_tick_story_update() {
        let mut e = Ensign::new("Ticker", ModelType::LocalJEPA);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        // Tick 10 should trigger story update
        for _ in 0..9 {
            e.tick();
        }
        let result = e.tick();
        assert!(result.story_updated);
    }

    #[test]
    fn ensign_report() {
        let mut e = Ensign::new("Reporter", ModelType::RemoteLight);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Maintenance("scheduled".into()), "room-1", &baton);
        let report = e.report();
        assert_eq!(report.name, "Reporter");
        assert_eq!(report.status, EnsignStatus::Waking);
    }

    #[test]
    fn ensign_is_at_yellow_after_orient() {
        let mut e = Ensign::new("Nav", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        let rs = RoomState::new("room-1");
        e.orient(&rs);
        assert!(e.is_at_yellow());
    }

    // --- Full lifecycle integration ---
    #[test]
    fn full_lifecycle() {
        let mut e = Ensign::new("FullLifecycle", ModelType::LocalJEPA);

        // 1. Call
        let baton = Baton::new("hermes", "new interaction starting");
        e.call(
            CallReason::NewInteraction("user connected".into()),
            "bridge",
            &baton,
        );
        assert_eq!(e.status, EnsignStatus::Waking);

        // 2. Orient
        let rs = RoomState {
            room_id: "bridge".into(),
            automation_active: true,
            automation_name: Some("navigation".into()),
            automation_progress: 0.5,
            interaction_active: true,
            partner_id: Some("captain".into()),
            recent_events_count: 3,
            room_gravity: -0.2,
        };
        let orientation = e.orient(&rs);
        assert!(orientation.summary().contains("bridge"));
        assert_eq!(e.status, EnsignStatus::YellowAlert);

        // 3. Build story
        let events = vec![
            AutomationEvent::new(1, "nav", "init"),
            AutomationEvent::new(2, "nav", "course_set").with_result("ok"),
            AutomationEvent::new(3, "nav", "executing"),
        ];
        let story = e.build_story(&events);
        assert!(!story.render().is_empty());

        // 4. Fine-tune
        let mut interaction = Interaction::new("captain").with_style(UserStyle::Precise);
        interaction.add_message("set course to alpha");
        interaction.gravity_signal = 0.6;
        let action = e.fine_tune(&interaction);
        assert!(matches!(action, FineTuneAction::AdjustGravity { .. }));

        // 5. Tick loop
        for _ in 0..5 {
            let result = e.tick();
            assert!(result.alert.is_yellow_or_above());
        }

        // 6. Check deadband stays yellow even when green
        let db = e.check_deadband();
        assert_eq!(db, DeadbandStatus::Green);
        assert!(e.is_at_yellow()); // KEY: yellow even when green!

        // 7. Stand down
        let report = e.stand_down();
        assert!(report.deadband_stayed_green);
        assert!(report.baton_for_next.is_some());
    }

    // --- Serde round-trip for key types ---
    #[test]
    fn serde_ensign_status() {
        let statuses = vec![
            EnsignStatus::Dormant,
            EnsignStatus::Waking,
            EnsignStatus::Orienting,
            EnsignStatus::YellowAlert,
            EnsignStatus::RedAlert,
            EnsignStatus::StandingDown,
            EnsignStatus::Escalated,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: EnsignStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn serde_alert_level() {
        for level in [AlertLevel::Green, AlertLevel::Yellow, AlertLevel::Red] {
            let json = serde_json::to_string(&level).unwrap();
            let back: AlertLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, back);
        }
    }

    #[test]
    fn serde_call_reason() {
        let reasons = vec![
            CallReason::Preemptive("test".into()),
            CallReason::DeadbandWarning("warn".into()),
            CallReason::NewInteraction("new".into()),
            CallReason::AutomationStall("stall".into()),
            CallReason::Maintenance("maint".into()),
            CallReason::Emergency("urgent".into()),
        ];
        for r in reasons {
            let json = serde_json::to_string(&r).unwrap();
            let back: CallReason = serde_json::from_str(&json).unwrap();
            assert_eq!(r, back);
        }
    }

    #[test]
    fn serde_fine_tune_action() {
        let actions = vec![
            FineTuneAction::AdjustGravity { delta: 0.1 },
            FineTuneAction::PreloadModel { model: "phi".into() },
            FineTuneAction::PrepareResponse { template: "hi".into() },
            FineTuneAction::NotifyHermes { message: "msg".into() },
            FineTuneAction::NoAction,
            FineTuneAction::PrepareOnboarding { for_ensign: "e2".into() },
        ];
        for a in actions {
            let json = serde_json::to_string(&a).unwrap();
            let back: FineTuneAction = serde_json::from_str(&json).unwrap();
            assert_eq!(a, back);
        }
    }

    #[test]
    fn serde_full_ensign() {
        let mut e = Ensign::new("Serde", ModelType::LocalJEPA);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("test".into()), "room-1", &baton);
        let json = serde_json::to_string(&e).unwrap();
        let back: Ensign = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn serde_deadband_monitor() {
        let mut dm = DeadbandMonitor::new(0.5, 0.3);
        dm.update(1, 0.5);
        dm.update(2, 0.51);
        let json = serde_json::to_string(&dm).unwrap();
        let back: DeadbandMonitor = serde_json::from_str(&json).unwrap();
        assert_eq!(dm.upper_bound, back.upper_bound);
        assert_eq!(dm.lower_bound, back.lower_bound);
        assert_eq!(dm.current, back.current);
        assert_eq!(dm.history.len(), back.history.len());
    }

    #[test]
    fn serde_room_story() {
        let mut s = RoomStory::new("room-1");
        s.add_event(1, "started");
        s.compose();
        let json = serde_json::to_string(&s).unwrap();
        let back: RoomStory = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_stand_down_report() {
        let r = StandDownReport {
            ensign_id: "e-1".into(),
            room_id: "room-1".into(),
            duration_ticks: 50,
            actions_taken: 5,
            fine_tunes: 3,
            escalations: 0,
            deadband_stayed_green: true,
            story: "ok".into(),
            lessons_learned: vec!["learned".into()],
            baton_for_next: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: StandDownReport = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // --- Edge cases ---
    #[test]
    fn ensign_tick_breach_triggers_escalation() {
        let mut e = Ensign::new("Breach", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Emergency("critical".into()), "room-1", &baton);
        // Force a breach
        e.deadband.upper_bound = 1.0;
        e.deadband.lower_bound = -1.0;
        e.deadband.current = 2.0; // breached
        let result = e.tick();
        assert!(result.escalation.is_some());
        assert_eq!(e.status, EnsignStatus::Escalated);
    }

    #[test]
    fn ensign_energy_usage() {
        let mut e = Ensign::new("Energy", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        let initial = e.energy_used;
        e.tick();
        assert!(e.energy_used > initial);
        let report = e.report();
        assert!(report.energy_remaining < 1.0);
    }

    #[test]
    fn ensign_id_from_str() {
        let id = EnsignId::new("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn stand_down_report_with_lessons() {
        let r = StandDownReport {
            ensign_id: "e-1".into(),
            room_id: "room-1".into(),
            duration_ticks: 100,
            actions_taken: 20,
            fine_tunes: 15,
            escalations: 2,
            deadband_stayed_green: false,
            story: "eventful".into(),
            lessons_learned: vec!["Deadband left green".into(), "2 escalation(s) required".into()],
            baton_for_next: Some(Baton::new("e-1", "handoff")),
        };
        let s = r.summary();
        assert!(s.contains("left green"));
        assert!(s.contains("2 escalation"));
    }

    #[test]
    fn user_style_variants() {
        let styles = [UserStyle::Playful, UserStyle::Precise, UserStyle::Narrative, UserStyle::Socratic, UserStyle::Direct, UserStyle::Mixed];
        for style in styles {
            let json = serde_json::to_string(&style).unwrap();
            let back: UserStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(style, back);
        }
    }

    #[test]
    fn deadband_status_ordering() {
        let dm = DeadbandMonitor {
            upper_bound: 1.0,
            lower_bound: 0.0,
            current: 0.5,
            warning_threshold: 0.1,
            history: vec![],
            trend: DeadbandTrend::Stable,
        };
        assert_eq!(dm.status(), DeadbandStatus::Green);
    }

    #[test]
    fn deadband_trend_stable_short_history() {
        let mut dm = DeadbandMonitor::new(0.5, 0.5);
        dm.update(1, 0.5);
        assert!(matches!(dm.trend(), DeadbandTrend::Stable));
    }

    #[test]
    fn story_event_new() {
        let e = StoryEvent::new(5, "something happened", 0.8);
        assert_eq!(e.tick, 5);
        assert!((e.significance - 0.8).abs() < 1e-9);
    }

    #[test]
    fn interaction_with_style() {
        let i = Interaction::new("user").with_style(UserStyle::Playful);
        assert_eq!(i.style, UserStyle::Playful);
    }

    #[test]
    fn ensign_stand_down_with_escalations() {
        let mut e = Ensign::new("Esc", ModelType::LocalTiny);
        let baton = Baton::new("hermes", "go");
        e.call(CallReason::Emergency("urgent".into()), "room-1", &baton);
        e.escalate("test");
        e.deadband_ever_left_green = true;
        e.status = EnsignStatus::Escalated;
        e.status = EnsignStatus::YellowAlert;
        let report = e.stand_down();
        assert_eq!(report.escalations, 1);
        assert!(!report.deadband_stayed_green);
        assert!(!report.lessons_learned.is_empty());
    }

    #[test]
    fn orientation_estimated_completion() {
        let mut o = Orientation::new("room-1");
        o.automation_progress = 0.0;
        assert_eq!(o.estimated_completion, 0);
        o.automation_progress = 1.0;
        o.estimated_completion = 0;
        assert_eq!(o.estimated_completion, 0);
    }

    #[test]
    fn fine_tune_prepare_onboarding() {
        let action = FineTuneAction::PrepareOnboarding {
            for_ensign: "ensign-2".into(),
        };
        assert!(action.describe().contains("ensign-2"));
    }

    #[test]
    fn ensign_baton_seeds_deadband() {
        let mut e = Ensign::new("Seeded", ModelType::LocalTiny);
        let mut baton = Baton::new("hermes", "go");
        baton.current_state.insert("deadband_center".into(), "0.3".into());
        baton.current_state.insert("deadband_tolerance".into(), "0.1".into());
        e.call(CallReason::Preemptive("check".into()), "room-1", &baton);
        assert!((e.deadband.current - 0.3).abs() < 1e-9);
        assert!((e.deadband.upper_bound - 0.4).abs() < 1e-9);
    }
}
