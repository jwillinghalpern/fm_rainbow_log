use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("already exists.")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("created and imported automatically.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("used instead since it refers to the same file.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Timestamp\tFilename\tError\tMessage")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" started")
    }
}
