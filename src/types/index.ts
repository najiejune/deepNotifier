export type NotificationMode = "Push" | "Pull" | "Both";
export type HttpMethod = "GET" | "POST";
export type WeekDay = "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun";
export type Severity = "Info" | "Warning" | "Critical";
export type TimerMode = "countdown" | "pomodoro";
export type TimerStatus = "idle" | "running" | "paused" | "completed";
export type PomodoroPhase = "work" | "short_break" | "long_break";
export type MarqueePosition = "Top" | "Bottom";

export type NotificationSource =
  | "GitHub"
  | "GitLab"
  | "Bitbucket"
  | "Custom"
  | { Poll: { endpoint_name: string } }
  | "Timer"
  | "Pomodoro"
  | "System";

export interface PollEndpoint {
  id: string;
  name: string;
  url: string;
  interval_secs: number;
  timeout_secs: number;
  method: HttpMethod;
  headers: Record<string, string>;
  body?: string;
  enabled: boolean;
}

export interface DndSchedule {
  id: string;
  name: string;
  start_time: string;
  end_time: string;
  days: WeekDay[];
  enabled: boolean;
}

export interface GeneralConfig {
  language: string;
  mode: NotificationMode;
  run_on_startup: boolean;
  minimize_to_tray: boolean;
  close_to_tray: boolean;
}

export interface WebhookConfig {
  enabled: boolean;
  port: number;
  secret: string;
  github_events: string[];
  gitlab_events: string[];
  bitbucket_events: string[];
  custom_enabled: boolean;
  custom_title_path: string;
  custom_body_path: string;
  custom_severity: string;
}

export interface PollConfig {
  enabled: boolean;
  endpoints: PollEndpoint[];
}

export interface NotificationConfig {
  sound_enabled: boolean;
  sound_file: string;
  sound_volume: number;
  marquee_enabled: boolean;
  tray_enabled: boolean;
  max_history: number;
}

export interface DndConfig {
  enabled: boolean;
  schedules: DndSchedule[];
}

export interface TimerConfig {
  pomodoro_work_mins: number;
  pomodoro_short_break_mins: number;
  pomodoro_long_break_mins: number;
  pomodoro_rounds: number;
  pomodoro_sound_file: string;
  auto_start_break: boolean;
  auto_start_work: boolean;
}

export interface MarqueeConfig {
  position: MarqueePosition;
  speed: number;
  height: number;
  font_size: number;
  font_family: string;
  icon_before: string;
  icon_after: string;
  bg_color: string;
  text_color: string;
  opacity: number;
  duration_secs: number;
}

export type TodoSource =
  | "Manual"
  | "Pull"
  | { Push: { remote_addr: string } };

export interface TodoItem {
  id: string;
  text: string;
  completed: boolean;
  due_date: string;
  created_at: string;
  source: TodoSource;
}

export interface TodoTimerConfig {
  workMins: number;
  shortBreakMins: number;
  longBreakMins: number;
  rounds: number;
}

export interface TodoPullEndpoint {
  id: string;
  name: string;
  enabled: boolean;
  url: string;
  interval_secs: number;
  method: HttpMethod;
  headers: Record<string, string>;
  body?: string;
}

export interface TodoConfig {
  pull_enabled: boolean;
  pull_endpoints: TodoPullEndpoint[];
  push_enabled: boolean;
  push_port: number;
}

export interface AppConfig {
  general: GeneralConfig;
  webhook: WebhookConfig;
  poll: PollConfig;
  notification: NotificationConfig;
  dnd: DndConfig;
  timer: TimerConfig;
  marquee: MarqueeConfig;
  todo: TodoConfig;
}

export interface NotificationEvent {
  id: string;
  source: NotificationSource;
  event_type: string;
  title: string;
  body: string;
  severity: Severity;
  timestamp: string;
  raw_payload?: unknown;
  url?: string;
}

export interface TimerState {
  mode: TimerMode;
  status: TimerStatus;
  remaining_secs: number;
  total_secs: number;
  pomodoro_round: number;
  pomodoro_phase?: PomodoroPhase;
  started_at?: string;
}
