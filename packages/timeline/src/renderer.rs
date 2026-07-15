use super::types::{TimelineAskHumanGroup, TimelineGroup, TimelineGroupData, TimelineHumanGroup};

pub trait TimelineRenderer {
    type Line;

    fn format_stats(&self, stats: &super::types::GroupStats) -> String;

    fn render_human_group(&self, group: &TimelineHumanGroup) -> Vec<Self::Line>;
    fn render_skill_group(&self, group: &TimelineGroupData) -> Vec<Self::Line>;
    fn render_ask_human_group(&self, group: &TimelineAskHumanGroup) -> Vec<Self::Line>;

    fn render_group(&self, group: &TimelineGroup) -> Vec<Self::Line> {
        match group {
            TimelineGroup::Human(g) => self.render_human_group(g),
            TimelineGroup::Skill(g) => self.render_skill_group(g),
            TimelineGroup::AskHuman(g) => self.render_ask_human_group(g),
        }
    }
}

pub trait TimelineOutput: TimelineRenderer {
    fn on_group_added(&mut self, index: usize, group: &TimelineGroup);
    fn on_group_updated(&mut self, index: usize, group: &TimelineGroup);
    fn on_group_finalized(&mut self, index: usize, group: &TimelineGroup);
}
