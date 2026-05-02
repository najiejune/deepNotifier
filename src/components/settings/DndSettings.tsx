import { useState } from "react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { Plus, Trash2, Pencil } from "lucide-react";
import { useI18n } from "@/i18n";
import type { DndConfig, DndSchedule, WeekDay } from "@/types";

interface Props {
  config: DndConfig;
  onChange: (config: DndConfig) => void;
}

function newSchedule(): DndSchedule {
  return {
    id: crypto.randomUUID(),
    name: "",
    start_time: "09:00",
    end_time: "17:00",
    days: ["Mon", "Tue", "Wed", "Thu", "Fri"],
    enabled: true,
  };
}

function ScheduleDialog({
  initial,
  onSave,
  onCancel,
}: {
  initial: DndSchedule;
  onSave: (s: DndSchedule) => void;
  onCancel: () => void;
}) {
  const t = useI18n();
  const [s, setS] = useState<DndSchedule>({ ...initial, days: [...initial.days] });

  const dayLabels: WeekDay[] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  const dayLabelMap: Record<WeekDay, string> = {
    Mon: t.settings.dnd.dayMon,
    Tue: t.settings.dnd.dayTue,
    Wed: t.settings.dnd.dayWed,
    Thu: t.settings.dnd.dayThu,
    Fri: t.settings.dnd.dayFri,
    Sat: t.settings.dnd.daySat,
    Sun: t.settings.dnd.daySun,
  };

  const toggleDay = (day: WeekDay) => {
    setS((prev) => ({
      ...prev,
      days: prev.days.includes(day) ? prev.days.filter((d) => d !== day) : [...prev.days, day],
    }));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onCancel} />
      <div className="relative bg-bg-base border border-border-default rounded-lg shadow-lg w-[420px] p-5 space-y-4">
        <p className="text-sm font-semibold">
          {initial.name ? t.settings.dnd.editSchedule : t.settings.dnd.addSchedule}
        </p>

        <Input
          placeholder={t.settings.dnd.scheduleName}
          value={s.name}
          onChange={(e) => setS({ ...s, name: e.target.value })}
        />

        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-text-secondary">{t.settings.dnd.startTime}</span>
            <Input type="time" value={s.start_time} onChange={(e) => setS({ ...s, start_time: e.target.value })} className="w-28" />
          </div>
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-text-secondary">{t.settings.dnd.endTime}</span>
            <Input type="time" value={s.end_time} onChange={(e) => setS({ ...s, end_time: e.target.value })} className="w-28" />
          </div>
        </div>

        <div>
          <p className="text-xs text-text-secondary mb-2">{t.settings.dnd.days}</p>
          <div className="flex gap-1.5">
            {dayLabels.map((day) => (
              <button
                key={day}
                onClick={() => toggleDay(day)}
                className={`w-8 h-8 rounded-full text-[11px] font-medium transition-colors ${
                  s.days.includes(day)
                    ? "bg-accent text-white"
                    : "bg-bg-layer border border-border-subtle text-text-muted hover:border-border-default"
                }`}
              >
                {dayLabelMap[day]}
              </button>
            ))}
          </div>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button size="sm" variant="ghost" onClick={onCancel}>{t.settings.dnd.cancel}</Button>
          <Button size="sm" onClick={() => onSave(s)}>{t.settings.dnd.save}</Button>
        </div>
      </div>
    </div>
  );
}

export function DndSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [dialog, setDialog] = useState<DndSchedule | null>(null);

  const setSchedules = (schedules: DndSchedule[]) =>
    onChange({ ...config, schedules });

  const handleSave = (s: DndSchedule) => {
    const exists = config.schedules.some((x) => x.id === s.id);
    if (exists) {
      setSchedules(config.schedules.map((x) => (x.id === s.id ? s : x)));
    } else {
      setSchedules([...config.schedules, s]);
    }
    setDialog(null);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.dnd.enable}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.dnd.enableDesc}</p>
        </div>
        <Toggle checked={config.enabled} onChange={(v) => onChange({ ...config, enabled: v })} />
      </div>

      <div className="border-t border-border-subtle pt-4 space-y-3">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium text-text">
            {t.settings.dnd.schedules}
          </p>
          <Button size="sm" variant="secondary" onClick={() => setDialog(newSchedule())}>
            <Plus size={12} /> {t.settings.dnd.addSchedule}
          </Button>
        </div>

        {config.schedules.map((sched) => (
          <div key={sched.id} className="bg-bg-layer border border-border-subtle rounded-sm px-3 py-2 flex items-center gap-3 group">
            <div className="flex-1 min-w-0">
              <p className="text-sm truncate">{sched.name || <span className="text-text-muted">Unnamed</span>}</p>
              <p className="text-[11px] font-mono text-text-muted">
                {sched.start_time} - {sched.end_time}
                <span className="ml-2">{sched.days.join(", ")}</span>
              </p>
            </div>
            <Toggle
              size="sm"
              checked={sched.enabled}
              onChange={(v) => {
                setSchedules(config.schedules.map((x) => (x.id === sched.id ? { ...x, enabled: v } : x)));
              }}
            />
            <Button size="sm" variant="ghost" onClick={() => setDialog(sched)}>
              <Pencil size={12} />
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setSchedules(config.schedules.filter((x) => x.id !== sched.id))}>
              <Trash2 size={12} />
            </Button>
          </div>
        ))}
      </div>

      {dialog && (
        <ScheduleDialog
          initial={dialog}
          onSave={handleSave}
          onCancel={() => setDialog(null)}
        />
      )}
    </div>
  );
}
