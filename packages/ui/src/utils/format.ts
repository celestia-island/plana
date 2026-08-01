/**
 * Canonical display-format helpers shared by celestia admin panels.
 * Consolidates the hand-rolled copies previously scattered across
 * arona (formatDate/formatUptime/formatNumber) and shittim-chest
 * (formatTokenCount/formatMediaTime).
 */

/** "1234" -> "1.2k", "2500000" -> "2.5M". */
export function formatTokenCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

/** "1234" -> "1.2k". */
export function formatNumber(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

/** Seconds -> "m:ss" (media player style). Negative/NaN clamp to 0. */
export function formatMediaTime(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) sec = 0;
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

/** Seconds -> "3h 12m" / "12m" / "45s". Negative/NaN clamp to 0. */
export function formatUptime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) seconds = 0;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${Math.floor(seconds)}s`;
}

/** "512" -> "512B", "1536" -> "1.5KB", "2621440" -> "2.5MB", "3221225472" -> "3.0GB". */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0B";
  if (bytes < 1024) return `${Math.floor(bytes)}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}GB`;
}

/** USD amount -> "$0.10" / "$1.5" / "$120". Negative/NaN clamp to 0. */
export function formatPriceUsd(n: number, currency = "$"): string {
  if (!Number.isFinite(n) || n < 0) n = 0;
  if (n >= 100) return `${currency}${Math.round(n)}`;
  if (n >= 1) return `${currency}${n.toFixed(1)}`;
  return `${currency}${n.toFixed(2)}`;
}

/** Timestamp -> "Just now" / "5m ago" / "3h ago" / "2d ago" / locale date. */
export function formatRelativeTime(input: string | number | Date): string {
  if (!input) return "";
  const d = input instanceof Date ? input : new Date(input);
  if (isNaN(d.getTime())) return "";
  const diff = Date.now() - d.getTime();
  const mins = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  if (mins < 1) return "Just now";
  if (mins < 60) return `${mins}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return d.toLocaleDateString();
}
