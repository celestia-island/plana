use std::marker::PhantomData;

use super::kind::AgentKind;

mod sealed {
    pub trait Sealed {}
}

pub trait AgentMarker: sealed::Sealed + 'static + Send + Sync + Copy + Clone {
    const KIND: AgentKind;
    const FOLDER_NAME: &'static str;
    const FRIENDLY_NAME: &'static str;
}

macro_rules! define_marker {
    ($($(#[$meta:meta])* $ident:ident => $kind:ident / $folder:literal / $friendly:literal),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $ident;

            impl sealed::Sealed for $ident {}

            impl AgentMarker for $ident {
                const KIND: AgentKind = AgentKind::$kind;
                const FOLDER_NAME: &'static str = $folder;
                const FRIENDLY_NAME: &'static str = $friendly;
            }
        )*
    };
}

define_marker! {
    HapLotesMarker => HapLotes / "haplotes" / "HapLotes",
    SkoPeoMarker => SkoPeo / "skopeo" / "SkoPeo",
    HubRisMarker => HubRis / "hubris" / "HubRis",
    KaLosMarker => KaLos / "kalos" / "KaLos",
    NeiKosMarker => NeiKos / "neikos" / "NeiKos",
    SkeMmaMarker => SkeMma / "skemma" / "SkeMma",
    ApoRiaMarker => ApoRia / "aporia" / "ApoRia",
    EleOsMarker => EleOs / "eleos" / "EleOs",
    EpieikeiaMarker => Epieikeia / "epieikeia" / "Epieikeia",
    OreXisMarker => OreXis / "orexis" / "OreXis",
    PhiLiaMarker => PhiLia / "philia" / "PhiLia",
    PoleMosMarker => PoleMos / "polemos" / "PoleMos",
    WebAutomationMarker => WebAutomation / "web_automation" / "Web Automation",
    ClassicSoftwareEngineeringMarker => ClassicSoftwareEngineering / "classic_software_engineering" / "Classic Software Engineering",
    IndustrialIoTMarker => IndustrialIoT / "industrial_iot" / "Industrial IoT",
    RemoteOperationsMarker => RemoteOperations / "remote_operations" / "Remote Operations",
}

#[derive(Debug, Clone, Copy)]
pub struct PhantomAgent<M: AgentMarker>(PhantomData<M>);

impl<M: AgentMarker> PhantomAgent<M> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    pub const fn kind(&self) -> AgentKind {
        M::KIND
    }

    pub const fn folder_name(&self) -> &'static str {
        M::FOLDER_NAME
    }

    pub const fn friendly_name(&self) -> &'static str {
        M::FRIENDLY_NAME
    }
}

impl<M: AgentMarker> Default for PhantomAgent<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_folder_matches_kind() {
        assert_eq!(
            HapLotesMarker::FOLDER_NAME,
            AgentKind::HapLotes.folder_name()
        );
        assert_eq!(SkoPeoMarker::FOLDER_NAME, AgentKind::SkoPeo.folder_name());
        assert_eq!(HubRisMarker::FOLDER_NAME, AgentKind::HubRis.folder_name());
        assert_eq!(
            WebAutomationMarker::FOLDER_NAME,
            AgentKind::WebAutomation.folder_name()
        );
    }

    #[test]
    fn phantom_agent_roundtrip() {
        let pa: PhantomAgent<HubRisMarker> = PhantomAgent::new();
        assert_eq!(pa.folder_name(), "hubris");
        assert_eq!(pa.friendly_name(), "HubRis");
        assert_eq!(pa.kind(), AgentKind::HubRis);
    }

    #[test]
    fn marker_is_zero_sized() {
        assert_eq!(std::mem::size_of::<HubRisMarker>(), 0);
        assert_eq!(std::mem::size_of::<PhantomAgent<HubRisMarker>>(), 0);
    }
}
