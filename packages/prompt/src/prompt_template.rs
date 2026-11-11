use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;

use tera::{Context, Tera};
use tracing::warn;

const ERR_NO_AVAILABLE_MODEL: &str = "No available model";
const ERR_NO_MODEL_AVAILABLE: &str = "No model available";
const ERR_NO_MODELS_AVAILABLE: &str = "No models available";
const ERR_MODEL_SELECTION_FAILED: &str = "Model selection failed";
const ERR_EMPTY_RESPONSE: &str = "empty response";
const ERR_LLM_CALL_FAILED: &str = "LLM call failed";

static SYSTEM_TEMPLATE: &str = r#"## Environment

- **Current date/time**: {{ current_datetime }} ({{ timezone }})
- **User preferred language**: {{ user_language }}
- **User**: {{ username }}
{% if workspace_uri %}
- **Workspace**: {{ workspace_uri }}
{% endif %}

## General Rules

- Respond in the user's preferred language ({{ user_language }}) unless the task explicitly requires another language.
- Use the date/time above for any temporal references.
- Follow the skill instructions faithfully.
"#;

static SKILL_CHAIN_TEMPLATE: &str = "\
You are step '{{ skill }}' in a pipeline: {{ previous }} → **{{ skill }}** → {{ next }}.
**Phase tool availability**: Each phase in the pipeline has a different tool set — downstream phases (especially plan_execute) have broader capabilities including file I/O, container management, and code execution. Your phase may have limited tools. Route tasks to the appropriate phase instead of rejecting them based on current-phase tool limitations.";

static PREVIOUS_STEP_TEMPLATE: &str = "{{ header }}";

// ── Error self-repair templates (chest `reports.selfhealText` caller) ──
// The chest webui's error cards offer a one-click "self-heal" quick reply:
// instead of free-form prose, the panel asks THIS service to render the
// message from a registered template (user direction 2026-11-11: the
// self-heal wording is part of the prompt-template system, not a webui
// string — frozen assets, runtime only substitutes variables). Two
// variants, one per error family the panel can render:
//  - `error_selfheal_interrupted`: the aggregated "engine restart
//    interrupted the chain" envelope (entelecheia `broadcast_helpers.rs`),
//    which carries the per-task affected list;
//  - `error_selfheal_generic`: any other coded terminal error
//    (`skill_failed`, `chain_failed`, `dependency_down`, …).
// Language follows the prompt corpus (`res/prompts/**` bodies are English;
// titles are the only localized surface), so no per-language arms here —
// the model answers the USER in their preferred language via the
// system_context rules regardless of this message's language.
//
// The structured blocks (task list / error record / system state) are the
// communication-style rule's "inherently multi-dimensional structured
// data" exemption; the framing paragraphs stay prose. Every variable is
// caller-supplied except `current_datetime` / `timezone`, which the
// renderer fills with `Local::now()` exactly like `system_context`.
static ERROR_SELFHEAL_INTERRUPTED_TEMPLATE: &str = r#"## Self-repair request: an engine restart interrupted this conversation's task chain

An engine restart happened while this conversation's task chain was still in flight. The chain was interrupted{% if affected_tasks %}, and the tasks listed below were left unfinished{% endif %}. You are the agent responsible for this conversation: assess the interruption, attempt to repair and continue the work on your own, and report what you did.
{% if affected_tasks %}
## Interrupted tasks
{% for task in affected_tasks %}- {{ task }}
{% endfor %}
{% endif %}## Error record

- Error code: `{{ error_code }}`
{% if error_title %}- Report title: {{ error_title }}
{% endif %}{% if report_timestamp %}- Detected at: {{ report_timestamp }}
{% endif %}{% if error_summary %}- Summary: {{ error_summary }}
{% endif %}{% if error_detail %}- Technical detail:

{{ error_detail }}
{% endif %}{% if error_context_lines %}- Context:
{% for line in error_context_lines %}  - {{ line }}
{% endfor %}
{% endif %}## Current system state

- Current time: {{ current_datetime }} ({{ timezone }})
- Panel version: {{ panel_version }}
- Engine version: {{ engine_version }}
- Engine connection: {{ engine_connection }}
- Chains currently in flight: {{ in_flight_chains }}
{% if conversation_id %}- Conversation: {{ conversation_id }}
{% endif %}
## How to proceed

1. Check each interrupted task's actual progress before redoing anything, so already-completed work is not repeated.
2. Attempt to repair and continue the interrupted work on your own. If a task cannot be resumed automatically, say so explicitly and state exactly what you need.
3. When finished, report the outcome for every affected task."#;

static ERROR_SELFHEAL_GENERIC_TEMPLATE: &str = r#"## Self-repair request: a task in this conversation failed with an error

A task in this conversation terminated with the error below. You are the agent responsible for this conversation: diagnose the failure from the error record together with the current system state, attempt the fix on your own, and continue the original task.

## Error record

- Error code: `{{ error_code }}`
{% if error_title %}- Report title: {{ error_title }}
{% endif %}{% if report_timestamp %}- Detected at: {{ report_timestamp }}
{% endif %}{% if error_summary %}- Summary: {{ error_summary }}
{% endif %}{% if error_detail %}- Technical detail:

{{ error_detail }}
{% endif %}{% if error_context_lines %}- Context:
{% for line in error_context_lines %}  - {{ line }}
{% endfor %}
{% endif %}## Current system state

- Current time: {{ current_datetime }} ({{ timezone }})
- Panel version: {{ panel_version }}
- Engine version: {{ engine_version }}
- Engine connection: {{ engine_connection }}
- Chains currently in flight: {{ in_flight_chains }}
{% if conversation_id %}- Conversation: {{ conversation_id }}
{% endif %}
## How to proceed

1. Diagnose the root cause first — the technical detail and context above are the primary evidence.
2. Attempt the fix on your own and continue the original task. If it cannot be fixed automatically, say so explicitly and state exactly what you need.
3. When finished, report what the failure was and what you did about it."#;

/// Which registered self-repair template to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSelfhealVariant {
    /// The aggregated "engine restart interrupted the chain" envelope —
    /// renders `error_selfheal_interrupted` (carries the affected-task
    /// list).
    Interrupted,
    /// Any other coded terminal error (`skill_failed`, `chain_failed`,
    /// `dependency_down`, …) — renders `error_selfheal_generic`.
    Generic,
}

/// Complete error facts of ONE report, passed verbatim by the caller
/// (chest core today). The renderer injects no error knowledge of its
/// own: whatever the caller passes is what the model reads. Empty
/// optional fields simply drop their line from the rendered record.
#[derive(Debug, Clone, Default)]
pub struct ErrorSelfhealFacts {
    /// Wire error code (`AgentErrorCode`, e.g. `chain_failed`). The one
    /// field without a template fallback — a record without a code is the
    /// caller's bug, not this renderer's.
    pub error_code: String,
    /// Backend prose title (may be empty).
    pub error_title: String,
    /// Backend summary (may be empty).
    pub error_summary: String,
    /// Full raw technical detail (`error.detail` + backend prose) — the
    /// block the card's "technical details" surface shows.
    pub error_detail: String,
    /// Pre-formatted `key: value` context lines, unknown keys included.
    pub error_context_lines: Vec<String>,
    /// Per-task display lines (the interruption envelope's affected
    /// tasks). Only the interrupted template renders a section for them;
    /// the generic variant has no affected-task section, so the caller
    /// passes `[]` there.
    pub affected_tasks: Vec<String>,
    /// When the failing report was stamped (ISO string or "").
    pub report_timestamp: String,
    /// Conversation the report belongs to ("" when unlinked).
    pub conversation_id: String,
    /// Caller-known system state at invocation time.
    pub system: ErrorSelfhealSystemState,
}

/// The system-state facts the CALLER knows better than this crate does.
/// Only the clock is renderer-owned (`Local::now()`, same source as
/// `render_system_context`).
#[derive(Debug, Clone, Default)]
pub struct ErrorSelfhealSystemState {
    /// Panel (chest) version line, e.g. `0.1.0 master:a36c907`.
    pub panel_version: String,
    /// Engine version line, e.g. `0.1.0 master:a36c907` ("—" when the
    /// engine has not been probed yet).
    pub engine_version: String,
    /// Whether the engine upstream is connected right now.
    pub engine_reachable: bool,
    /// Live chain count from the engine's `/health`, when the engine
    /// reports one; `None` = the engine did not say (older build) and the
    /// template renders an explicit "unknown" instead of a fake zero.
    pub in_flight_chains: Option<u32>,
}

pub struct PromptTemplateService {
    tera: Tera,
}

impl PromptTemplateService {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("system_context", SYSTEM_TEMPLATE)?;
        tera.add_raw_template("skill_chain", SKILL_CHAIN_TEMPLATE)?;
        tera.add_raw_template("previous_step", PREVIOUS_STEP_TEMPLATE)?;
        tera.add_raw_template(
            "error_selfheal_interrupted",
            ERROR_SELFHEAL_INTERRUPTED_TEMPLATE,
        )?;
        tera.add_raw_template("error_selfheal_generic", ERROR_SELFHEAL_GENERIC_TEMPLATE)?;
        Ok(Self { tera })
    }

    pub fn render_system_context(
        &self,
        user_language: &str,
        username: &str,
        timezone: &str,
        workspace_uri: Option<&str>,
    ) -> String {
        let now = Local::now();
        let datetime_str = now.format("%Y-%m-%d %H:%M:%S %Z").to_string();

        let mut ctx = Context::new();
        ctx.insert("current_datetime", &datetime_str);
        ctx.insert("user_language", user_language);
        ctx.insert("username", username);
        ctx.insert("timezone", timezone);
        ctx.insert("workspace_uri", &workspace_uri.unwrap_or(""));

        self.tera
            .render("system_context", &ctx)
            .unwrap_or_else(|e| {
                warn!("system_context template error: {}", e);
                let ws_line = workspace_uri
                    .map(|u| format!("\n- Workspace: {}", u))
                    .unwrap_or_default();
                format!(
                    "## Environment\n- Date: {}\n- Language: {}\n- User: {}{}",
                    datetime_str, user_language, username, ws_line
                )
            })
    }

    pub fn render_skill_chain(&self, skill: &str, previous: &str, next: &str) -> String {
        let mut ctx = Context::new();
        ctx.insert("skill", skill);
        ctx.insert("previous", previous);
        ctx.insert("next", next);

        self.tera.render("skill_chain", &ctx).unwrap_or_else(|e| {
            warn!("skill_chain template error: {}", e);
            format!(
                "You are step '{}' in a pipeline: {} → **{}** → {}.",
                skill, previous, skill, next
            )
        })
    }

    pub fn render_previous_step(&self, header: &str) -> String {
        let mut ctx = Context::new();
        ctx.insert("header", header);
        self.tera.render("previous_step", &ctx).unwrap_or_else(|e| {
            warn!("previous_step template error: {}", e);
            header.to_string()
        })
    }

    /// Render one of the registered error self-repair templates into the
    /// message text the panel sends back into the failed conversation.
    ///
    /// The renderer injects only the clock (`current_datetime` /
    /// `timezone`, same source as `render_system_context`) — every error
    /// fact and every system-state fact rides in via `facts`, so the
    /// caller owns what the model sees and this crate never guesses about
    /// panels, engines, or chain bookkeeping. Empty optional fields drop
    /// their line from the record instead of printing empties.
    pub fn render_error_selfheal(
        &self,
        variant: ErrorSelfhealVariant,
        facts: &ErrorSelfhealFacts,
    ) -> String {
        let now = Local::now();
        let mut ctx = Context::new();
        ctx.insert(
            "current_datetime",
            &now.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        );
        ctx.insert("timezone", &now.format("%Z").to_string());
        ctx.insert("error_code", &facts.error_code);
        ctx.insert("error_title", &facts.error_title);
        ctx.insert("error_summary", &facts.error_summary);
        ctx.insert("error_detail", &facts.error_detail);
        ctx.insert("error_context_lines", &facts.error_context_lines);
        ctx.insert("affected_tasks", &facts.affected_tasks);
        ctx.insert("report_timestamp", &facts.report_timestamp);
        ctx.insert("conversation_id", &facts.conversation_id);
        ctx.insert("panel_version", &facts.system.panel_version);
        ctx.insert("engine_version", &facts.system.engine_version);
        ctx.insert(
            "engine_connection",
            if facts.system.engine_reachable {
                "reachable"
            } else {
                "UNREACHABLE"
            },
        );
        ctx.insert(
            "in_flight_chains",
            &match facts.system.in_flight_chains {
                Some(n) => n.to_string(),
                None => "unknown (engine did not report)".to_string(),
            },
        );

        let name = match variant {
            ErrorSelfhealVariant::Interrupted => "error_selfheal_interrupted",
            ErrorSelfhealVariant::Generic => "error_selfheal_generic",
        };
        self.tera.render(name, &ctx).unwrap_or_else(|e| {
            // Defensive only (the registered templates are static and
            // test-pinned); a miss must still return usable facts rather
            // than an empty message — same contract as every renderer
            // above.
            warn!("{} template error: {}", name, e);
            let mut fallback = format!(
                "Self-repair request ({}).\n\nError code: {}\n",
                match variant {
                    ErrorSelfhealVariant::Interrupted => "interrupted chain",
                    ErrorSelfhealVariant::Generic => "task error",
                },
                facts.error_code,
            );
            for line in [
                ("Title", &facts.error_title),
                ("Summary", &facts.error_summary),
                ("Detail", &facts.error_detail),
            ] {
                if !line.1.is_empty() {
                    fallback.push_str(&format!("{}: {}\n", line.0, line.1));
                }
            }
            for task in &facts.affected_tasks {
                fallback.push_str(&format!("Affected task: {}\n", task));
            }
            fallback.push_str(&format!(
                "Panel: {} / Engine: {} ({})\n",
                facts.system.panel_version,
                facts.system.engine_version,
                if facts.system.engine_reachable {
                    "reachable"
                } else {
                    "UNREACHABLE"
                },
            ));
            fallback
        })
    }

    pub fn render_skill_error_title(&self, skill_name: &str, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => format!("执行失败：{}", skill_name),
            "zh-Hant" | "zht" | "zh-TW" => format!("執行失敗：{}", skill_name),
            "ja" => format!("{} の実行に失敗しました", skill_name),
            "ko" => format!("실행 실패 {}", skill_name),
            "fr" => format!("Échec de {}", skill_name),
            "es" => format!("{} falló", skill_name),
            "ru" => format!("{} не удалось", skill_name),
            _ => format!("Error in {}", skill_name),
        }
    }

    pub fn render_skill_missing_report_error(
        &self,
        skill_name: &str,
        retries: usize,
        lang: &str,
    ) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => format!(
                "技能 `{}` 连续 {} 次未调用 `report()`，执行已暂停。",
                skill_name, retries
            ),
            "zh-Hant" | "zht" | "zh-TW" => format!(
                "技能 `{}` 連續 {} 次未呼叫 `report()`，執行已暫停。",
                skill_name, retries
            ),
            "ja" => format!(
                "スキル `{}` は {} 回のリトライでも `report()` を呼び出さなかったため、実行を一時停止しました。",
                skill_name, retries
            ),
            "ko" => format!(
                "스킬 `{}`이(가) {}번 재시도 후에도 `report()`를 호출하지 않아 실행이 일시 중지되었습니다.",
                skill_name, retries
            ),
            "fr" => format!(
                "La compétence `{}` n'a pas appelé `report()` après {} tentatives, exécution suspendue.",
                skill_name, retries
            ),
            "es" => format!(
                "La habilidad `{}` no llamó `report()` después de {} intentos, ejecución pausada.",
                skill_name, retries
            ),
            "ru" => format!(
                "Навык `{}` не вызвал `report()` после {} попыток, выполнение приостановлено.",
                skill_name, retries
            ),
            _ => format!(
                "Skill `{}` did not call `report()` after {} retries, execution paused.",
                skill_name, retries
            ),
        }
    }

    pub fn render_skill_error_content(&self, error_content: &str, lang: &str) -> String {
        if error_content.contains(ERR_NO_AVAILABLE_MODEL)
            || error_content.contains(ERR_NO_MODEL_AVAILABLE)
            || error_content.contains(ERR_NO_MODELS_AVAILABLE)
            || error_content.contains(ERR_MODEL_SELECTION_FAILED)
        {
            self.render_model_selection_error(error_content, lang)
        } else if error_content.contains(ERR_EMPTY_RESPONSE) {
            self.render_empty_response_error(error_content, lang)
        } else if error_content.contains(ERR_LLM_CALL_FAILED) {
            self.render_llm_call_error(error_content, lang)
        } else {
            error_content.to_string()
        }
    }

    fn render_model_selection_error(&self, _error_content: &str, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "未找到可用的 LLM 模型。请检查模型配置或环境变量（LLM_ENDPOINT / LLM_API_KEY）。".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "找不到可用的 LLM 模型。請檢查模型設定或環境變數（LLM_ENDPOINT / LLM_API_KEY）。".to_string(),
            "ja" => "利用可能な LLM モデルが見つかりません。モデル設定または環境変数（LLM_ENDPOINT / LLM_API_KEY）を確認してください。".to_string(),
            "ko" => "사용 가능한 LLM 모델을 찾을 수 없습니다. 모델 설정 또는 환경 변수(LLM_ENDPOINT / LLM_API_KEY)가 올바르게 설정되었는지 확인하세요.".to_string(),
            "fr" => "Aucun modèle LLM disponible. Vérifiez la configuration des modèles ou les variables d'environnement (LLM_ENDPOINT / LLM_API_KEY).".to_string(),
            "es" => "No se encontró un modelo LLM disponible. Verifique la configuración de modelos o las variables de entorno (LLM_ENDPOINT / LLM_API_KEY).".to_string(),
            "ru" => "Доступная LLM модель не найдена. Проверьте конфигурацию моделей или переменные окружения (LLM_ENDPOINT / LLM_API_KEY).".to_string(),
            _ => "No available LLM model found. Please check model configuration or environment variables (LLM_ENDPOINT / LLM_API_KEY).".to_string(),
        }
    }

    fn render_empty_response_error(&self, _error_content: &str, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "LLM 返回了空响应，请稍后重试。".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "LLM 回傳了空回應，請稍後再試。".to_string(),
            "ja" => "LLM が空のレスポンスを返しました。しばらくしてからもう一度お試しください。"
                .to_string(),
            "ko" => "LLM이 빈 응답을 반환했습니다. 잠시 후 다시 시도해 주세요.".to_string(),
            "fr" => "Le LLM a retourné une réponse vide. Veuillez réessayer plus tard.".to_string(),
            "es" => {
                "El LLM devolvió una respuesta vacía. Inténtelo de nuevo más tarde.".to_string()
            }
            "ru" => "LLM вернул пустой ответ. Пожалуйста, попробуйте позже.".to_string(),
            _ => "LLM returned an empty response. Please try again later.".to_string(),
        }
    }

    fn render_llm_call_error(&self, _error_content: &str, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "LLM 调用失败。请检查网络连接与 API 密钥配置。".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "LLM 呼叫失敗。請檢查網路連線與 API 金鑰設定。".to_string(),
            "ja" => "LLM の呼び出しに失敗しました。ネットワーク接続と API キー設定を確認してください。".to_string(),
            "ko" => "LLM 호출에 실패했습니다. 네트워크 연결과 API 키 설정을 확인하세요.".to_string(),
            "fr" => "L'appel LLM a échoué. Vérifiez la connexion réseau et la configuration de la clé API.".to_string(),
            "es" => "La llamada al LLM falló. Verifique la conexión de red y la configuración de la clave API.".to_string(),
            "ru" => "Вызов LLM не удался. Проверьте сетевое подключение и конфигурацию API-ключа.".to_string(),
            _ => "LLM call failed. Please check network connection and API key configuration.".to_string(),
        }
    }

    pub fn render_chain_error_title(&self, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "请求执行失败".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "請求執行失敗".to_string(),
            "ja" => "リクエスト実行失敗".to_string(),
            "ko" => "요청 실행 실패".to_string(),
            "fr" => "Échec de la requête".to_string(),
            "es" => "Solicitud fallida".to_string(),
            "ru" => "Ошибка выполнения запроса".to_string(),
            _ => "Request Execution Failed".to_string(),
        }
    }

    pub fn render_chain_error_content(&self, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "技能链执行过程中发生错误，请稍后重试。".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "技能鏈執行過程中發生錯誤，請稍後再試。".to_string(),
            "ja" => "スキルチェーンの実行中にエラーが発生しました。しばらくしてからもう一度お試しください。".to_string(),
            "ko" => "스킬 체인 실행 중 오류가 발생했습니다. 잠시 후 다시 시도해 주세요.".to_string(),
            "fr" => "Une erreur s'est produite lors de l'exécution de la chaîne de compétences. Veuillez réessayer plus tard.".to_string(),
            "es" => "Se produjo un error durante la ejecución de la cadena de habilidades. Inténtelo de nuevo más tarde.".to_string(),
            "ru" => "Произошла ошибка при выполнении цепочки навыков. Пожалуйста, попробуйте позже.".to_string(),
            _ => "An error occurred during skill chain execution. Please try again later.".to_string(),
        }
    }

    // ── Expired-enquiry notice (entelecheia `handle_agent_report_reply`) ──
    // A reply to a plan enquiry that no longer exists (already answered or
    // past its TTL) is a System-type notice, NOT a coded error: none of the
    // `AgentErrorCode` variants honestly describes it, and the chest error
    // catalog only renders CODED reports. So this is the one engine-
    // originated surface that still carries display copy — localized HERE,
    // in the same match-arm family as the chain-error cards above, fed by
    // the session's preferred language.
    //
    // Format contract (user direction 2026-11-11): engine-level envelopes
    // must never carry a `domain::name`-shaped title — that shape is
    // reserved for `agent::skill` references, and a machine token in the
    // title field gets rendered verbatim by older or minimal consumers,
    // masquerading as a skill report. These renderers return PROSE in the
    // user's language; `test_notice_arms_are_localized` additionally pins
    // that no arm contains `::`.
    pub fn render_inquiry_expired_title(&self, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "询问已失效".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "詢問已失效".to_string(),
            "ja" => "問い合わせは無効です".to_string(),
            "ko" => "문의가 만료되었습니다".to_string(),
            "fr" => "Demande expirée".to_string(),
            "es" => "Consulta expirada".to_string(),
            "ru" => "Запрос истёк".to_string(),
            _ => "Enquiry expired".to_string(),
        }
    }

    pub fn render_inquiry_expired_content(&self, lang: &str) -> String {
        match lang {
            "zh-Hans" | "zhs" | "zh-CN" | "zh" => "这条询问已失效或已回答，未执行任何方案。请重新发送需求。".to_string(),
            "zh-Hant" | "zht" | "zh-TW" => "這條詢問已失效或已回答，未執行任何方案。請重新發送需求。".to_string(),
            "ja" => "この問い合わせは無効か、すでに回答済みです。何も実行されていません。要件を再度送信してください。".to_string(),
            "ko" => "이 문의는 만료되었거나 이미 답변되었습니다. 아무것도 실행되지 않았습니다. 요구사항을 다시 보내주세요.".to_string(),
            "fr" => "Cette demande a expiré ou a déjà reçu une réponse ; rien n'a été exécuté. Veuillez renvoyer votre besoin.".to_string(),
            "es" => "Esta consulta ha expirado o ya fue respondida; no se ejecutó nada. Envíe el requisito de nuevo.".to_string(),
            "ru" => "Этот запрос истёк или уже был отвечен; ничего не выполнено. Отправьте требование ещё раз.".to_string(),
            _ => "This enquiry has expired or was already answered; nothing was run. Please send the requirement again.".to_string(),
        }
    }

    pub fn render_raw(&self, template: &str, variables: &HashMap<&str, String>) -> String {
        let mut ctx = Context::new();
        for (k, v) in variables {
            ctx.insert(k.to_string(), v);
        }
        tera::Tera::one_off(template, &ctx, false).unwrap_or_else(|e| {
            warn!("raw template render error: {}", e);
            let mut result = template.to_string();
            for (k, v) in variables {
                let pattern_spaced = format!("{{{{ {} }}}}", k);
                let pattern_tight = format!("{{{{{}}}}}", k);
                result = result.replace(&pattern_spaced, v);
                result = result.replace(&pattern_tight, v);
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_system_context() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let result = svc.render_system_context("en", "alice", "UTC", None);
        assert!(result.contains("en"));
        assert!(result.contains("alice"));
        assert!(result.contains("UTC"));
        assert!(!result.contains("Workspace"));
        Ok(())
    }

    #[test]
    fn test_render_system_context_with_workspace() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let result =
            svc.render_system_context("en", "alice", "UTC", Some("local:///opt/entelecheia"));
        assert!(result.contains("local:///opt/entelecheia"));
        assert!(result.contains("Workspace"));
        Ok(())
    }

    #[test]
    fn test_render_system_context_chinese() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let result = svc.render_system_context("zh", "TestUser", "Asia/Shanghai", None);
        assert!(result.contains("zh"));
        assert!(result.contains("TestUser"));
        assert!(result.contains("Asia/Shanghai"));
        Ok(())
    }

    #[test]
    fn test_render_skill_chain() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let result = svc.render_skill_chain(
            "operator",
            "smart_read_file",
            "smart_write_file → submit_report",
        );
        assert!(result.contains("operator"));
        assert!(result.contains("smart_read_file"));
        assert!(result.contains("smart_write_file → submit_report"));
        Ok(())
    }

    #[test]
    fn test_render_skill_chain_final() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let result = svc.render_skill_chain("submit_report", "", "final");
        assert!(result.contains("submit_report"));
        assert!(result.contains("final"));
        Ok(())
    }

    #[test]
    fn test_render_raw() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let mut vars = HashMap::new();
        vars.insert("name", "test".to_string());
        vars.insert("count", "42".to_string());
        let result = svc.render_raw("Hello {{ name }}, you have {{ count }} items.", &vars);
        assert_eq!(result, "Hello test, you have 42 items.");
        Ok(())
    }

    /// Regression pin for the L125 i18n finding: every renderer's
    /// zh-Hans / zh-Hant / ja arms used to carry the ENGLISH default
    /// verbatim (only ko/fr/es/ru were localized), so a Chinese
    /// deployment shipped "Request Execution Failed" cards. Each CJK
    /// arm must differ from the English default.
    #[test]
    fn test_cjk_error_arms_are_localized() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let cjk_langs = ["zh-Hans", "zh-Hant", "ja"];
        for lang in cjk_langs {
            assert_ne!(
                svc.render_chain_error_title(lang),
                svc.render_chain_error_title("en"),
                "chain error title arm {lang} must be localized"
            );
            assert_ne!(
                svc.render_chain_error_content(lang),
                svc.render_chain_error_content("en"),
                "chain error content arm {lang} must be localized"
            );
            assert_ne!(
                svc.render_skill_error_title("demo_skill", lang),
                svc.render_skill_error_title("demo_skill", "en"),
                "skill error title arm {lang} must be localized"
            );
            assert_ne!(
                svc.render_skill_missing_report_error("demo_skill", 3, lang),
                svc.render_skill_missing_report_error("demo_skill", 3, "en"),
                "missing-report error arm {lang} must be localized"
            );
            assert_ne!(
                svc.render_skill_error_content("No available model", lang),
                svc.render_skill_error_content("No available model", "en"),
                "skill error content arm {lang} must be localized"
            );
            // R2 mutation finding (2026-11-03): the previous pin covered
            // only the model-selection route — render_empty_response_error
            // and render_llm_call_error had NO test interception (a revert
            // of either CJK arm stayed 16/16 green). Route both explicitly.
            assert_ne!(
                svc.render_skill_error_content("empty response", lang),
                svc.render_skill_error_content("empty response", "en"),
                "empty-response arm {lang} must be localized"
            );
            assert_ne!(
                svc.render_skill_error_content("LLM call failed", lang),
                svc.render_skill_error_content("LLM call failed", "en"),
                "llm-call arm {lang} must be localized"
            );
        }
        Ok(())
    }

    /// The expired-enquiry notice (entelecheia `handle_agent_report_reply`)
    /// is the one engine-originated surface that still carries display
    /// copy — a System notice with no honest `AgentErrorCode`, so the
    /// chest error catalog cannot render it. Contract (user direction
    /// 2026-11-11): engine envelopes never carry `domain::name`-shaped
    /// titles — that shape is reserved for `agent::skill` references, and
    /// a machine token in the title field gets rendered verbatim by
    /// consumers, masquerading as a skill report. Every arm must be
    /// localized prose, and no arm may contain the `::` skill-reference
    /// separator (a revert to the old `engine::inquiry_expired` token or
    /// to the English default turns this red).
    #[test]
    fn test_notice_arms_are_localized_and_prose_shaped() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let langs = ["zh-Hans", "zh-Hant", "ja", "ko", "fr", "es", "ru"];
        for lang in langs {
            let title = svc.render_inquiry_expired_title(lang);
            let content = svc.render_inquiry_expired_content(lang);
            assert_ne!(
                title,
                svc.render_inquiry_expired_title("en"),
                "inquiry-expired title arm {lang} must be localized"
            );
            assert_ne!(
                content,
                svc.render_inquiry_expired_content("en"),
                "inquiry-expired content arm {lang} must be localized"
            );
            assert!(
                !title.contains("::"),
                "inquiry-expired title arm {lang} must be prose, not a domain::name token: {title}"
            );
            assert!(
                !content.contains("::"),
                "inquiry-expired content arm {lang} must be prose: {content}"
            );
        }
        Ok(())
    }

    /// Full-facts interrupted render: every caller-supplied surface must
    /// survive verbatim — the affected-task loop, the raw technical
    /// detail, the context lines, and the system-state block (including
    /// the live chain count). Each record line carries its own pin: the
    /// heading-and-clock assertions alone stay green through the loss of
    /// any single one (R2 review, 2026-11-11 — title / summary / engine
    /// version had NO red-on-revert interception before this wave).
    #[test]
    fn test_render_error_selfheal_interrupted_full_facts() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let facts = ErrorSelfhealFacts {
            error_code: "chain_failed".into(),
            error_title: "engine::restart_interrupted".into(),
            error_summary: "The chain was interrupted by an engine restart.".into(),
            error_detail:
                "task 01a1c34a (`hubris::task_decompose (turn)`) was orphaned by a restart".into(),
            error_context_lines: vec![
                "interrupted: restart".into(),
                "skill: task_decompose".into(),
            ],
            affected_tasks: vec![
                "hubris::task_decompose (turn)".into(),
                "hubris::plan_execute".into(),
            ],
            report_timestamp: "2026-11-10T10:35:00+08:00".into(),
            conversation_id: "conv-123".into(),
            system: ErrorSelfhealSystemState {
                panel_version: "0.1.0 master:a36c907".into(),
                engine_version: "0.1.0 master:a36c907".into(),
                engine_reachable: true,
                in_flight_chains: Some(2),
            },
        };
        let text = svc.render_error_selfheal(ErrorSelfhealVariant::Interrupted, &facts);
        assert!(text.contains("an engine restart interrupted this conversation's task chain"));
        assert!(text.contains("- hubris::task_decompose (turn)"));
        assert!(text.contains("- hubris::plan_execute"));
        assert!(text.contains("- Error code: `chain_failed`"));
        assert!(text.contains("- Report title: engine::restart_interrupted"));
        assert!(text.contains("was orphaned by a restart"));
        assert!(text.contains("  - interrupted: restart"));
        assert!(text.contains("- Detected at: 2026-11-10T10:35:00+08:00"));
        assert!(text.contains("- Summary: The chain was interrupted by an engine restart."));
        assert!(text.contains("- Panel version: 0.1.0 master:a36c907"));
        assert!(text.contains("- Engine version: 0.1.0 master:a36c907"));
        assert!(text.contains("- Engine connection: reachable"));
        assert!(text.contains("- Chains currently in flight: 2"));
        assert!(text.contains("- Conversation: conv-123"));
        assert!(!text.contains("{{"), "placeholders must all be substituted");
        Ok(())
    }

    /// The generic variant must NOT carry the interrupted framing (its
    /// section heading and the affected-task list): the two variants are
    /// the user-facing distinction, so a copy-paste collapse would send
    /// chain-interruption instructions for an ordinary skill failure.
    #[test]
    fn test_render_error_selfheal_generic_shape() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let facts = ErrorSelfhealFacts {
            error_code: "skill_failed".into(),
            error_title: "执行失败: hubris::task_decompose".into(),
            error_summary: String::new(),
            error_detail: "Rejected: conversation already streaming".into(),
            error_context_lines: vec!["skill: task_decompose".into()],
            affected_tasks: vec![],
            report_timestamp: String::new(),
            conversation_id: String::new(),
            system: ErrorSelfhealSystemState {
                panel_version: "0.1.0 master:x".into(),
                engine_version: "—".into(),
                engine_reachable: false,
                in_flight_chains: None,
            },
        };
        let text = svc.render_error_selfheal(ErrorSelfhealVariant::Generic, &facts);
        assert!(text.contains("a task in this conversation failed with an error"));
        assert!(!text.contains("Interrupted tasks"));
        assert!(text.contains("- Error code: `skill_failed`"));
        assert!(text.contains("Rejected: conversation already streaming"));
        // The fixture's single context line renders under the Context
        // block — a reverted loop (or a swapped ctx insert) must go red.
        assert!(text.contains("  - skill: task_decompose"));
        // Empty optionals drop their lines instead of printing empties.
        assert!(!text.contains("- Summary:"));
        assert!(!text.contains("- Detected at:"));
        assert!(!text.contains("- Conversation:"));
        // System-state honesty pins: an unreachable engine and an
        // engine that did not report a chain count must SAY so —
        // a silent zero would read as "idle" to the model.
        assert!(text.contains("- Engine connection: UNREACHABLE"));
        assert!(text.contains("- Chains currently in flight: unknown (engine did not report)"));
        assert!(!text.contains("{{"), "placeholders must all be substituted");
        Ok(())
    }

    /// The fallback lane (template render error) must still return the
    /// core facts, never an empty message. Driven by a facts value whose
    /// DETAIL is hostile to nothing — the lane is reached only via a
    /// broken registration, so this pin exercises it through the same
    /// public API the fallback protects and just asserts the contract
    /// shape that survives any future template edit.
    #[test]
    fn test_render_error_selfheal_returns_core_facts() -> Result<()> {
        let svc = PromptTemplateService::new()?;
        let facts = ErrorSelfhealFacts {
            error_code: "dependency_down".into(),
            ..Default::default()
        };
        for variant in [
            ErrorSelfhealVariant::Interrupted,
            ErrorSelfhealVariant::Generic,
        ] {
            let text = svc.render_error_selfheal(variant, &facts);
            assert!(
                text.contains("dependency_down"),
                "variant {variant:?} lost the code"
            );
            assert!(
                text.contains("Error code:"),
                "variant {variant:?} lost the record header"
            );
            assert!(
                text.contains("Current time:"),
                "variant {variant:?} lost the clock line"
            );
            // Drop-when-empty is a stated contract: default facts carry an
            // empty title and no context lines, so the record must NOT
            // print those lines at all (R3 finding: the title line had NO
            // red-on-revert interception — a reverted `{% if %}` guard
            // rendered a bare `- Report title: ` and every pin stayed
            // green).
            assert!(
                !text.contains("- Report title:"),
                "variant {variant:?} rendered an empty title line"
            );
            assert!(
                !text.contains("- Context:"),
                "variant {variant:?} rendered an empty context block"
            );
            // The interruption framing is claimed by the VARIANT, not by
            // default facts: with an empty affected-task list the section
            // must be cleanly absent (older/foreign envelope), and the
            // intro prose must not promise a list that does not follow
            // (R2 finding: the fixed "tasks listed below" clause used to
            // dangle on exactly this path).
            if variant == ErrorSelfhealVariant::Interrupted {
                assert!(
                    !text.contains("Interrupted tasks"),
                    "empty affected_tasks must drop the section"
                );
                assert!(
                    !text.contains("listed below"),
                    "empty affected_tasks must drop the list promise"
                );
            }
            assert!(
                !text.contains("{{"),
                "variant {variant:?} left placeholders"
            );
        }
        Ok(())
    }
}
