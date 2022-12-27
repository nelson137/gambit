use bevy::prelude::*;

pub trait AppPushOrderedStartupStages {
    fn push_ordered_startup_stages<I: IntoIterator<Item = (impl StageLabel + Clone, SystemStage)>>(
        &mut self,
        labels_and_stages: I,
    ) -> &mut Self;
}

impl AppPushOrderedStartupStages for App {
    fn push_ordered_startup_stages<
        I: IntoIterator<Item = (impl StageLabel + Clone, SystemStage)>,
    >(
        &mut self,
        labels_and_stages: I,
    ) -> &mut Self {
        let mut labels_and_stages = labels_and_stages.into_iter();
        let first = labels_and_stages.next().expect("list is empty");
        self.add_startup_stage_after(StartupStage::Startup, first.0.clone(), first.1);
        labels_and_stages.into_iter().fold(first.0, |last_label, (label, stage)| {
            self.add_startup_stage_after(last_label, label.clone(), stage);
            label
        });
        self
    }
}
