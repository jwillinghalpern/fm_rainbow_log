use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("redan finns.")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("skapades och importerades automatiskt.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("används i stället, eftersom den hänvisar till samma fil.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Tidsstämpel	Filnamn	Fel	Meddelande")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" startats")
    }
}
