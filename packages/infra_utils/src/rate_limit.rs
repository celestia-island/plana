//! 通用多窗口限流器（固定窗口计数 + 可选令牌配额）。
//!
//! 2026-10-20 上游化：scepter 网关（512 行手搓）与 chest（channel/ingress/mfa 三份
//! 平行实现）此前各自维护固定窗口计数器。本模块提供唯一的通用核心——按 key 维护
//! 任意多档窗口，每档同时携带「请求数上限」与可选「令牌配额」，突发系数按档设置；
//! 黑/白名单属调用方策略，不进核心。时钟可注入，测试确定性（不睡真实时间）。

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

/// 单档窗口的配额定义。
#[derive(Debug, Clone)]
pub struct WindowSpec {
    /// 窗口名（诊断用，如 `"minute"` / `"hour"` / `"day"`）。
    pub name: &'static str,
    /// 窗口长度。
    pub window: Duration,
    /// 每窗口请求数上限。
    pub count_limit: u64,
    /// 每窗口令牌配额（如 LLM token 预算）；`None` = 只限请求数。
    pub token_limit: Option<u64>,
    /// 突发系数（对 count_limit 生效；1.0 = 无突发余量）。
    pub burst_multiplier: f64,
}

impl WindowSpec {
    pub fn new(name: &'static str, window: Duration, count_limit: u64) -> Self {
        Self {
            name,
            window,
            count_limit,
            token_limit: None,
            burst_multiplier: 1.0,
        }
    }

    /// 附带令牌配额。
    pub fn with_tokens(mut self, token_limit: u64) -> Self {
        self.token_limit = Some(token_limit);
        self
    }

    /// 附带突发系数。
    pub fn with_burst(mut self, multiplier: f64) -> Self {
        self.burst_multiplier = multiplier;
        self
    }

    fn effective_count_limit(&self) -> u64 {
        (self.count_limit as f64 * self.burst_multiplier) as u64
    }
}

/// 一次 `check` 的裁决。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitDecision {
    /// 是否放行（放行即计数已同步累加）。
    pub allowed: bool,
    /// 拒绝时：距最先违反的窗口重置还有多久。
    pub retry_after: Option<Duration>,
    /// 拒绝时：最先违反的窗口名。
    pub violated: Option<&'static str>,
}

impl RateLimitDecision {
    fn allow() -> Self {
        Self {
            allowed: true,
            retry_after: None,
            violated: None,
        }
    }

    fn deny(name: &'static str, retry_after: Duration) -> Self {
        Self {
            allowed: false,
            retry_after: Some(retry_after),
            violated: Some(name),
        }
    }
}

#[derive(Debug)]
struct WindowState {
    count: u64,
    tokens: u64,
    start: Instant,
}

impl WindowState {
    fn new(now: Instant) -> Self {
        Self {
            count: 0,
            tokens: 0,
            start: now,
        }
    }

    /// 窗口过期则重置；返回 `(仍处有效期内, 剩余时长)`。
    fn roll(&mut self, window: Duration, now: Instant) -> (bool, Duration) {
        let elapsed = now.duration_since(self.start);
        if elapsed >= window {
            self.count = 0;
            self.tokens = 0;
            self.start = now;
            (false, Duration::ZERO)
        } else {
            (true, window - elapsed)
        }
    }
}

type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

/// 按 key 的多窗口限流器；`check` 原子完成「判罚 + 计数」（拒绝不计数）。
pub struct RateLimiter<K: Eq + Hash + Clone> {
    specs: Vec<WindowSpec>,
    slots: RwLock<HashMap<K, Vec<WindowState>>>,
    clock: Clock,
}

impl<K: Eq + Hash + Clone> RateLimiter<K> {
    pub fn new(specs: Vec<WindowSpec>) -> Self {
        assert!(!specs.is_empty(), "rate limiter needs at least one window");
        Self {
            specs,
            slots: RwLock::new(HashMap::new()),
            clock: Arc::new(Instant::now),
        }
    }

    /// 注入时钟（测试专用；生产一律用真实时钟）。
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// 判定并记账：`token_cost` 计入每档窗口的令牌配额（无配额档忽略）。
    ///
    /// 语义 = scepter 网关原手搓实现：先按序滚动各窗口，再对所有限额做判定，
    /// 任一档超限即整体拒绝（**拒绝不计数**），`retry_after` 取最先违反档的剩余时长。
    pub async fn check(&self, key: &K, token_cost: u64) -> RateLimitDecision {
        let now = (self.clock)();
        let mut slots = self.slots.write().await;
        let states = slots
            .entry(key.clone())
            .or_insert_with(|| self.specs.iter().map(|_| WindowState::new(now)).collect());
        debug_assert_eq!(states.len(), self.specs.len());

        // 第一遍：滚动窗口 + 预判（不写入）。
        let mut violation: Option<(&'static str, Duration)> = None;
        let mut projected: Vec<(u64, u64)> = Vec::with_capacity(self.specs.len());
        for (spec, state) in self.specs.iter().zip(states.iter_mut()) {
            let (_active, remaining) = state.roll(spec.window, now);
            let next_count = state.count + 1;
            let next_tokens = state.tokens + token_cost;
            let over = next_count > spec.effective_count_limit()
                || spec.token_limit.is_some_and(|limit| next_tokens > limit);
            if over && violation.is_none() {
                violation = Some((spec.name, remaining));
            }
            projected.push((next_count, next_tokens));
        }

        if let Some((name, retry_after)) = violation {
            return RateLimitDecision::deny(name, retry_after);
        }

        // 第二遍：放行，落账。
        for ((count, tokens), state) in projected.into_iter().zip(states.iter_mut()) {
            state.count = count;
            state.tokens = tokens;
        }
        RateLimitDecision::allow()
    }

    /// 只读查询某 key 指定窗口的 `(剩余请求数, 距重置时长)`：不记账、不改动状态；
    /// key 尚无记录 → `None`（与原手搓实现的 get_remaining 语义一致）；窗口已过
    /// 有效期 → `(满额, 0)`。
    pub async fn remaining(&self, key: &K, window: &str) -> Option<(u64, Duration)> {
        let now = (self.clock)();
        let idx = self.specs.iter().position(|spec| spec.name == window)?;
        let spec = &self.specs[idx];
        let slots = self.slots.read().await;
        let state = slots.get(key)?.get(idx)?;
        let elapsed = now.duration_since(state.start);
        if elapsed >= spec.window {
            Some((spec.effective_count_limit(), Duration::ZERO))
        } else {
            let left = spec.effective_count_limit().saturating_sub(state.count);
            Some((left, spec.window - elapsed))
        }
    }

    /// 清掉全部窗口均已空闲超过 `max_idle` 的 key（定期巡检用；`scepter` 的
    /// cleanup_interval 语义）。返回清除的 key 数。
    pub async fn retain_active(&self, max_idle: Duration) -> usize {
        let now = (self.clock)();
        let mut slots = self.slots.write().await;
        let before = slots.len();
        slots.retain(|_, states| {
            states
                .iter()
                .zip(self.specs.iter())
                .any(|(state, spec)| now.duration_since(state.start) < spec.window + max_idle)
        });
        before - slots.len()
    }

    /// 当前被跟踪的 key 数（监控/测试）。
    pub async fn tracked_keys(&self) -> usize {
        self.slots.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn minute_hour() -> Vec<WindowSpec> {
        vec![
            WindowSpec::new("minute", Duration::from_secs(60), 2),
            WindowSpec::new("hour", Duration::from_secs(3600), 5),
        ]
    }

    /// 可拨动的假时钟。
    struct FakeClock(AtomicU64);
    impl FakeClock {
        fn new() -> Arc<Self> {
            Arc::new(Self(AtomicU64::new(0)))
        }
        fn advance(&self, secs: u64) {
            self.0.fetch_add(secs, Ordering::SeqCst);
        }
    }

    fn limiter<K: Eq + Hash + Clone>(specs: Vec<WindowSpec>) -> (RateLimiter<K>, Arc<FakeClock>) {
        let clock = FakeClock::new();
        let c = clock.clone();
        (
            RateLimiter::new(specs).with_clock(Arc::new(move || {
                Instant::now() - Duration::from_secs(10_000)
                    + Duration::from_secs(c.0.load(Ordering::SeqCst))
            })),
            clock,
        )
    }

    #[tokio::test]
    async fn allows_under_limit_and_denies_on_minute_cap() {
        let (l, _) = limiter::<&str>(minute_hour());
        assert!(l.check(&"a", 0).await.allowed);
        assert!(l.check(&"a", 0).await.allowed);
        let d = l.check(&"a", 0).await;
        assert!(!d.allowed, "minute cap of 2 must deny the 3rd request");
        assert_eq!(d.violated, Some("minute"));
        assert!(d.retry_after.unwrap() > Duration::ZERO);
    }

    #[tokio::test]
    async fn minute_window_resets_but_hour_accumulates() {
        let (l, clock) = limiter::<&str>(minute_hour());
        // 两个完整分钟窗各打满 2 次：分钟窗每轮翻页归零，小时窗持续累计到 4。
        for cycle in 0..2 {
            if cycle > 0 {
                clock.advance(61);
            }
            assert!(l.check(&"a", 0).await.allowed);
            assert!(l.check(&"a", 0).await.allowed);
        }
        clock.advance(61); // 第三个分钟窗
        assert!(
            l.check(&"a", 0).await.allowed,
            "minute=1 fine, hour=5 at cap"
        );
        let d = l.check(&"a", 0).await;
        assert!(
            !d.allowed,
            "hour cap of 5 must bind (5+1 > 5) while minute has room"
        );
        assert_eq!(d.violated, Some("hour"));
    }

    #[tokio::test]
    async fn denial_does_not_count() {
        // 双窗设计（R3 配方）：minute 上限 1、hour 上限 2。允许 1 次、拒绝 1 次后，
        // 推进恰好 60s 让 minute 翻页而 hour 仍在窗内——
        //   正确语义：hour 只记 1 次，第 3 次 check 的 hour 投影 2 ≤ 2 → 放行；
        //   「拒绝也计数」变异：hour 已记 2 次，投影 3 > 2 → 拒绝（测试变红）。
        // （前版锚用 advance(60) 落在单窗边界，roll 先归零再判，变异遗留计数被抹掉，
        //   两实现同绿——已被 R3 变异实证推翻。）
        let (l, clock) = limiter::<&str>(vec![
            WindowSpec::new("minute", Duration::from_secs(60), 1),
            WindowSpec::new("hour", Duration::from_secs(3600), 2),
        ]);
        assert!(l.check(&"k", 0).await.allowed, "first request passes");
        assert!(!l.check(&"k", 0).await.allowed, "minute cap 1 denies");
        clock.advance(60); // minute 翻页归零；hour（elapsed 60 < 3600）仍在窗内
        assert!(
            l.check(&"k", 0).await.allowed,
            "denials must not count: hour carries only the allowed hit (2 <= 2); a counting implementation would sit at 3 > 2 and deny"
        );
    }

    #[tokio::test]
    async fn token_quota_denies_independently_of_count() {
        let (l, _) = limiter::<&str>(vec![
            WindowSpec::new("m", Duration::from_secs(60), 100).with_tokens(10),
        ]);
        assert!(l.check(&"a", 6).await.allowed);
        let d = l.check(&"a", 6).await;
        assert!(!d.allowed, "token budget must deny before the count cap");
        assert_eq!(d.violated, Some("m"));
    }

    #[tokio::test]
    async fn burst_multiplier_widens_the_count_cap_only() {
        // 突发系数只放大请求数档（2→4），不碰令牌档（10）：前三次各耗 2 token
        // （累计 6，请求数 3 ≤ 4），第四次请求数 4 ≤ 4 仍过、令牌 6+5=11 > 10 拒。
        let (l, _) = limiter::<&str>(vec![
            WindowSpec::new("m", Duration::from_secs(60), 2)
                .with_burst(2.0)
                .with_tokens(10),
        ]);
        assert!(l.check(&"a", 2).await.allowed);
        assert!(l.check(&"a", 2).await.allowed);
        assert!(
            l.check(&"a", 2).await.allowed,
            "burst 2x lifts the count cap 2 -> 4"
        );
        let d = l.check(&"a", 5).await;
        assert!(!d.allowed, "token cap stays at 10 (6+5 > 10)");
        assert_eq!(d.violated, Some("m"));
    }

    #[tokio::test]
    async fn keys_are_isolated() {
        let (l, _) = limiter::<&str>(vec![WindowSpec::new("m", Duration::from_secs(60), 1)]);
        assert!(l.check(&"alice", 0).await.allowed);
        assert!(!l.check(&"alice", 0).await.allowed);
        assert!(l.check(&"bob", 0).await.allowed, "bob is a fresh key");
    }

    #[tokio::test]
    async fn retry_after_reports_the_violating_windows_remaining_time() {
        let (l, clock) = limiter::<&str>(vec![WindowSpec::new("m", Duration::from_secs(60), 1)]);
        l.check(&"k", 0).await;
        clock.advance(10);
        let d = l.check(&"k", 0).await;
        assert!(!d.allowed);
        let ra = d.retry_after.unwrap();
        assert!(
            ra <= Duration::from_secs(50) && ra > Duration::from_secs(49),
            "≈50s left, got {ra:?}"
        );
    }

    #[tokio::test]
    async fn remaining_reports_left_quota_and_absent_keys() {
        let (l, clock) = limiter::<&str>(minute_hour());
        assert_eq!(
            l.remaining(&"ghost", "minute").await,
            None,
            "absent key -> None"
        );
        l.check(&"a", 0).await;
        l.check(&"a", 0).await;
        let (left, reset_in) = l.remaining(&"a", "minute").await.unwrap();
        assert_eq!(left, 0, "minute cap 2 fully spent");
        assert!(reset_in > Duration::ZERO && reset_in <= Duration::from_secs(60));
        let (hour_left, _) = l.remaining(&"a", "hour").await.unwrap();
        assert_eq!(hour_left, 3, "hour cap 5 spent 2");
        clock.advance(61);
        let (left, reset_in) = l.remaining(&"a", "minute").await.unwrap();
        assert_eq!(
            (left, reset_in),
            (2, Duration::ZERO),
            "expired window reports full quota"
        );
    }

    #[tokio::test]
    async fn retain_active_drops_idle_keys_only() {
        let (l, clock) = limiter::<&str>(minute_hour());
        l.check(&"hot", 0).await;
        l.check(&"cold", 0).await;
        clock.advance(3600 + 60); // 两档窗口都早已翻页
        l.check(&"hot", 0).await; // hot 重新活跃
        clock.advance(30);
        let dropped = l.retain_active(Duration::from_secs(10)).await;
        assert_eq!(dropped, 1, "only the idle key drops");
        assert_eq!(l.tracked_keys().await, 1);
    }
}
