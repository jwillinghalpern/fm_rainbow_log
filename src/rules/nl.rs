use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("bestaat.")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("is automatisch gemaakt en geïmporteerd.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("worden gebruikt omdat deze naar hetzelfde bestand verwijst.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Tijdstempel	Bestandsnaam	Fout	Bericht")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" gestart")
    }
}
