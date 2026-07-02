import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";

export function AppFilter() {
  const { t } = useI18n();
  const apps = useAppStore((s) => s.apps);
  const selectedAppId = useAppStore((s) => s.selectedAppId);
  const setSelectedAppId = useAppStore((s) => s.setSelectedAppId);

  if (apps.length === 0) return null;

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-surface)] overflow-x-auto">
      <button
        onClick={() => setSelectedAppId(null)}
        className={`px-3 py-1 text-xs rounded-full whitespace-nowrap transition-colors ${
          selectedAppId === null
            ? "bg-[var(--color-primary)] text-white"
            : "bg-[var(--color-bg)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
        }`}
      >
        {t.filter.allApps}
      </button>
      {apps.map((app) => (
        <button
          key={app.appid}
          onClick={() => setSelectedAppId(app.appid)}
          className={`px-3 py-1 text-xs rounded-full whitespace-nowrap transition-colors ${
            selectedAppId === app.appid
              ? "bg-[var(--color-primary)] text-white"
              : "bg-[var(--color-bg)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
          }`}
        >
          {app.appname}
        </button>
      ))}
    </div>
  );
}
