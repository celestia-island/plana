use tracing::info;

use super::{
    super::{
        renderer::{TimelineOutput, TimelineRenderer},
        types::{
            GroupState, GroupStats, SkillBlockStatus, TimelineAskHumanGroup, TimelineGroup,
            TimelineGroupData, TimelineHumanGroup,
        },
    },
    CliTimelineRenderer,
};

pub struct CliTimelineBuffer {
    renderer: CliTimelineRenderer,
    groups: Vec<(TimelineGroup, bool)>,
}

impl Default for CliTimelineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CliTimelineBuffer {
    pub fn new() -> Self {
        Self {
            renderer: CliTimelineRenderer,
            groups: Vec::new(),
        }
    }

    pub fn next_index(&self) -> usize {
        self.groups.len()
    }

    pub fn add_group(&mut self, group: TimelineGroup) {
        let state = group.state();
        let should_print = state == GroupState::Finalized;
        let idx = self.groups.len();
        self.groups.push((group, false));
        if should_print {
            self.flush_group(idx);
        }
    }

    pub fn update_group(&mut self, index: usize, group: TimelineGroup) {
        if index < self.groups.len() {
            self.groups[index].0 = group;
        }
    }

    pub fn get_skill_data(&self, index: usize) -> Option<&TimelineGroupData> {
        self.groups.get(index).and_then(|(g, _)| match g {
            TimelineGroup::Skill(data) => Some(&**data),
            _ => None,
        })
    }

    pub fn find_unfinalized_skill(
        &self,
        agent_type: Option<&str>,
        agent_number: Option<&str>,
        skill_name: Option<&str>,
    ) -> Option<usize> {
        self.groups
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (g, printed))| {
                if *printed {
                    return false;
                }
                match g {
                    TimelineGroup::Skill(data) => {
                        if data.state == GroupState::Finalized {
                            return false;
                        }
                        if agent_type.is_some_and(|t| data.agent_type != t) {
                            return false;
                        }
                        if agent_number.is_some_and(|n| data.agent_number != n) {
                            return false;
                        }
                        if let Some(sn) = skill_name {
                            let matches = data
                                .skill_name
                                .as_ref()
                                .is_some_and(|stored| stored.ends_with(sn));
                            if !matches {
                                return false;
                            }
                        }
                        true
                    }
                    _ => false,
                }
            })
            .map(|(idx, _)| idx)
    }

    pub fn finalize_preceding_skill(&mut self, agent_number: &str) {
        if let Some(idx) = self.find_unfinalized_skill(None, Some(agent_number), None) {
            self.finalize_group(idx);
        }
    }

    pub fn finalize_group_with_stats(&mut self, index: usize, stats: GroupStats, summary: String) {
        if index < self.groups.len() {
            self.groups[index].0 = match &self.groups[index].0 {
                TimelineGroup::Skill(data) => {
                    let mut d = data.clone();
                    d.state = GroupState::Finalized;
                    d.stats = Some(stats);
                    d.summary = Some(summary);
                    TimelineGroup::Skill(d)
                }
                other => other.clone(),
            };
            self.flush_group(index);
        }
    }

    pub fn finalize_group(&mut self, index: usize) {
        if index < self.groups.len() {
            self.groups[index].0 = match &self.groups[index].0 {
                TimelineGroup::Skill(data) => {
                    let mut d = data.clone();
                    d.state = GroupState::Finalized;
                    d.status = SkillBlockStatus::Done;
                    TimelineGroup::Skill(d)
                }
                other => other.clone(),
            };
            self.flush_group(index);
        }
    }

    fn flush_group(&mut self, index: usize) {
        if index < self.groups.len() && !self.groups[index].1 {
            let lines = self.renderer.render_group(&self.groups[index].0);
            for line in lines {
                info!("{}", line);
            }
            self.groups[index].1 = true;
        }
    }
}

impl TimelineRenderer for CliTimelineBuffer {
    type Line = String;

    fn format_stats(&self, stats: &GroupStats) -> String {
        self.renderer.format_stats(stats)
    }

    fn render_human_group(&self, group: &TimelineHumanGroup) -> Vec<String> {
        self.renderer.render_human_group(group)
    }

    fn render_skill_group(&self, group: &TimelineGroupData) -> Vec<String> {
        self.renderer.render_skill_group(group)
    }

    fn render_ask_human_group(&self, group: &TimelineAskHumanGroup) -> Vec<String> {
        self.renderer.render_ask_human_group(group)
    }
}

impl TimelineOutput for CliTimelineBuffer {
    fn on_group_added(&mut self, index: usize, group: &TimelineGroup) {
        if index < self.groups.len() {
            return;
        }
        let should_print = group.state() == GroupState::Finalized;
        self.groups.push((group.clone(), false));
        if should_print {
            self.flush_group(index);
        }
    }

    fn on_group_updated(&mut self, _index: usize, _group: &TimelineGroup) {}

    fn on_group_finalized(&mut self, index: usize, _group: &TimelineGroup) {
        self.flush_group(index);
    }
}
