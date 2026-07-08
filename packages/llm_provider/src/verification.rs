use anyhow::Result;

use arona_config::GenProtocol;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentIntegrity {
    pub protocol: GenProtocol,
    pub signatures: Vec<ContentSignature>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentSignature {
    pub kind: SignatureKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureKind {
    EncryptedReasoning,
    ThinkingSignature,
    RedactedThinking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    NotSupported,
    Verified,
    Failed,
}

pub const VERIFICATION_REJECT_MESSAGE: &str = "MCP tool call rejected: content integrity verification failed — \
     the response may have been tampered with in transit. \
     Refusing to execute tool to protect system integrity.";

pub trait ContentVerification {
    fn verification_status(
        protocol: GenProtocol,
        integrity: Option<&ContentIntegrity>,
    ) -> VerificationStatus;
}

impl ContentVerification for GenProtocol {
    fn verification_status(
        protocol: GenProtocol,
        integrity: Option<&ContentIntegrity>,
    ) -> VerificationStatus {
        match protocol {
            GenProtocol::OpenAIResponsesV1 => match integrity {
                None => VerificationStatus::NotSupported,
                Some(integrity) => {
                    let has_encrypted = integrity.signatures.iter().any(|s| {
                        s.kind == SignatureKind::EncryptedReasoning && !s.value.is_empty()
                    });
                    if has_encrypted {
                        VerificationStatus::Verified
                    } else {
                        VerificationStatus::NotSupported
                    }
                },
            },
            GenProtocol::AnthropicMessagesV1 | GenProtocol::AnthropicMessagesV2 => {
                match integrity {
                    None => VerificationStatus::NotSupported,
                    Some(integrity) => {
                        let has_signature = integrity.signatures.iter().any(|s| {
                            matches!(s.kind, SignatureKind::ThinkingSignature)
                                && !s.value.is_empty()
                        });
                        let has_redacted = integrity
                            .signatures
                            .iter()
                            .any(|s| s.kind == SignatureKind::RedactedThinking);
                        if has_signature || has_redacted {
                            VerificationStatus::Verified
                        } else {
                            VerificationStatus::NotSupported
                        }
                    },
                }
            },
            GenProtocol::OpenAIChatV1 | GenProtocol::GeminiGenerateV1 => {
                VerificationStatus::NotSupported
            },
            // Non-LLM protocols don't use content verification
            _ => VerificationStatus::NotSupported,
        }
    }
}

impl ContentIntegrity {
    pub fn new(protocol: GenProtocol) -> Self {
        Self {
            protocol,
            signatures: Vec::new(),
        }
    }

    pub fn with_signature(mut self, kind: SignatureKind, value: String) -> Self {
        self.signatures.push(ContentSignature { kind, value });
        self
    }

    pub fn verify(&self) -> VerificationStatus {
        GenProtocol::verification_status(self.protocol, Some(self))
    }

    pub fn check_tool_call(
        protocol: GenProtocol,
        integrity: Option<&ContentIntegrity>,
    ) -> Result<(), &'static str> {
        let Some(integrity) = integrity else {
            return Ok(());
        };

        if integrity.protocol != protocol {
            return Err(VERIFICATION_REJECT_MESSAGE);
        }

        match integrity.verify() {
            VerificationStatus::Verified => Ok(()),
            VerificationStatus::NotSupported => Ok(()),
            VerificationStatus::Failed => Err(VERIFICATION_REJECT_MESSAGE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_chat_v1_not_supported() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIChatV1);
        assert_eq!(integrity.verify(), VerificationStatus::NotSupported);
        Ok(())
    }

    #[test]
    fn test_gemini_not_supported() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::GeminiGenerateV1);
        assert_eq!(integrity.verify(), VerificationStatus::NotSupported);
        Ok(())
    }

    #[test]
    fn test_openai_responses_no_signatures() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIResponsesV1);
        assert_eq!(integrity.verify(), VerificationStatus::NotSupported);
        Ok(())
    }

    #[test]
    fn test_openai_responses_with_encrypted_reasoning() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIResponsesV1)
            .with_signature(SignatureKind::EncryptedReasoning, "sig123".to_string());
        assert_eq!(integrity.verify(), VerificationStatus::Verified);
        Ok(())
    }

    #[test]
    fn test_openai_responses_empty_encrypted_reasoning() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIResponsesV1)
            .with_signature(SignatureKind::EncryptedReasoning, String::new());
        assert_eq!(integrity.verify(), VerificationStatus::NotSupported);
        Ok(())
    }

    #[test]
    fn test_anthropic_no_signatures() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::AnthropicMessagesV1);
        assert_eq!(integrity.verify(), VerificationStatus::NotSupported);
        Ok(())
    }

    #[test]
    fn test_anthropic_with_thinking_signature() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::AnthropicMessagesV1)
            .with_signature(SignatureKind::ThinkingSignature, "sig".to_string());
        assert_eq!(integrity.verify(), VerificationStatus::Verified);
        Ok(())
    }

    #[test]
    fn test_anthropic_with_redacted_thinking() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::AnthropicMessagesV1)
            .with_signature(SignatureKind::RedactedThinking, String::new());
        assert_eq!(integrity.verify(), VerificationStatus::Verified);
        Ok(())
    }

    #[test]
    fn test_check_tool_call_none_integrity() -> Result<()> {
        assert!(ContentIntegrity::check_tool_call(GenProtocol::OpenAIChatV1, None).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_tool_call_protocol_mismatch() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::AnthropicMessagesV1);
        let result = ContentIntegrity::check_tool_call(GenProtocol::OpenAIChatV1, Some(&integrity));
        assert_eq!(result.unwrap_err(), VERIFICATION_REJECT_MESSAGE);
        Ok(())
    }

    #[test]
    fn test_check_tool_call_verified() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIResponsesV1)
            .with_signature(SignatureKind::EncryptedReasoning, "sig".to_string());
        assert!(
            ContentIntegrity::check_tool_call(GenProtocol::OpenAIResponsesV1, Some(&integrity))
                .is_ok()
        );
        Ok(())
    }

    #[test]
    fn test_check_tool_call_not_supported_ok() -> Result<()> {
        let integrity = ContentIntegrity::new(GenProtocol::OpenAIChatV1);
        assert!(
            ContentIntegrity::check_tool_call(GenProtocol::OpenAIChatV1, Some(&integrity)).is_ok()
        );
        Ok(())
    }
}
