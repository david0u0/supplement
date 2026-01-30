use crate::SupplementID;

#[derive(Debug)]
pub enum SingleHistory {
    Flag { id: SupplementID, value: String },
    Command(SupplementID),
    Arg { id: SupplementID, value: String },
}
#[derive(Default, Debug)]
pub struct History(Vec<SingleHistory>);
impl History {
    pub fn push_arg(&mut self, id: SupplementID, value: String) {
        self.0.push(SingleHistory::Arg { id, value });
    }
    pub fn push_flag(&mut self, id: SupplementID, value: String) {
        self.0.push(SingleHistory::Flag { id, value });
    }
    pub fn push_pure_flag(&mut self, id: SupplementID) {
        self.0.push(SingleHistory::Flag {
            id,
            value: String::new(),
        });
    }
    pub fn push_command(&mut self, id: SupplementID) {
        self.0.push(SingleHistory::Command(id))
    }

    pub fn find(&self, id: SupplementID) -> Option<&SingleHistory> {
        self.0.iter().find(|h| {
            let cur_id = match h {
                SingleHistory::Arg { id, .. } => id,
                SingleHistory::Flag { id, .. } => id,
                SingleHistory::Command(id) => id,
            };
            *cur_id == id
        })
    }
}
