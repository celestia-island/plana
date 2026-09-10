use strum::AsRefStr;

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ReplParamKey {
    Code,
    Path,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ToolCallParamKey {
    ToolName,
    Parameters,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ResponseKey {
    Success,
    Error,
    Data,
    Stdout,
    Path,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ToolListToolsResultKey {
    Tools,
    Count,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ContainerCreateParamKey {
    Image,
    Name,
    Env,
    Ports,
    Network,
    Volumes,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ContainerVolumeKey {
    HostPath,
    ContainerPath,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ContainerForkParamKey {
    ContainerId,
    Name,
    NamespaceVolume,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
pub enum BridgeKey {
    #[strum(serialize = "type")]
    Type,
    #[strum(serialize = "data")]
    Data,
    #[strum(serialize = "action")]
    Action,
}

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum AuthParamKey {
    DelegatorId,
    Reason,
    TargetLevel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn repl_param_keys() -> Result<()> {
        assert_eq!(ReplParamKey::Code.as_ref(), "code");
        assert_eq!(ReplParamKey::Path.as_ref(), "path");
        Ok(())
    }

    #[test]
    fn tool_call_param_keys() -> Result<()> {
        assert_eq!(ToolCallParamKey::ToolName.as_ref(), "tool_name");
        assert_eq!(ToolCallParamKey::Parameters.as_ref(), "parameters");
        Ok(())
    }

    #[test]
    fn response_keys() -> Result<()> {
        assert_eq!(ResponseKey::Success.as_ref(), "success");
        assert_eq!(ResponseKey::Error.as_ref(), "error");
        assert_eq!(ResponseKey::Data.as_ref(), "data");
        assert_eq!(ResponseKey::Stdout.as_ref(), "stdout");
        assert_eq!(ResponseKey::Path.as_ref(), "path");
        Ok(())
    }

    #[test]
    fn tool_list_tools_result_keys() -> Result<()> {
        assert_eq!(ToolListToolsResultKey::Tools.as_ref(), "tools");
        assert_eq!(ToolListToolsResultKey::Count.as_ref(), "count");
        Ok(())
    }

    #[test]
    fn container_create_param_keys() -> Result<()> {
        assert_eq!(ContainerCreateParamKey::Image.as_ref(), "image");
        assert_eq!(ContainerCreateParamKey::Name.as_ref(), "name");
        assert_eq!(ContainerCreateParamKey::Env.as_ref(), "env");
        assert_eq!(ContainerCreateParamKey::Ports.as_ref(), "ports");
        assert_eq!(ContainerCreateParamKey::Network.as_ref(), "network");
        assert_eq!(ContainerCreateParamKey::Volumes.as_ref(), "volumes");
        Ok(())
    }

    #[test]
    fn container_volume_keys() -> Result<()> {
        assert_eq!(ContainerVolumeKey::HostPath.as_ref(), "host_path");
        assert_eq!(ContainerVolumeKey::ContainerPath.as_ref(), "container_path");
        assert_eq!(ContainerVolumeKey::ReadOnly.as_ref(), "read_only");
        Ok(())
    }

    #[test]
    fn bridge_keys() -> Result<()> {
        assert_eq!(BridgeKey::Type.as_ref(), "type");
        assert_eq!(BridgeKey::Data.as_ref(), "data");
        assert_eq!(BridgeKey::Action.as_ref(), "action");
        Ok(())
    }

    #[test]
    fn container_fork_param_keys() -> Result<()> {
        assert_eq!(ContainerForkParamKey::ContainerId.as_ref(), "container_id");
        assert_eq!(ContainerForkParamKey::Name.as_ref(), "name");
        assert_eq!(
            ContainerForkParamKey::NamespaceVolume.as_ref(),
            "namespace_volume"
        );
        Ok(())
    }

    #[test]
    fn auth_param_keys() -> Result<()> {
        assert_eq!(AuthParamKey::DelegatorId.as_ref(), "delegator_id");
        assert_eq!(AuthParamKey::Reason.as_ref(), "reason");
        assert_eq!(AuthParamKey::TargetLevel.as_ref(), "target_level");
        Ok(())
    }
}
