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

pub struct PromptTemplateService {
    tera: Tera,
}

impl PromptTemplateService {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("system_context", SYSTEM_TEMPLATE)?;
        tera.add_raw_template("skill_chain", SKILL_CHAIN_TEMPLATE)?;
        tera.add_raw_template("previous_step", PREVIOUS_STEP_TEMPLATE)?;
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
        }
        Ok(())
    }
}
