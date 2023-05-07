use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("già esistente.") || msg.contains("esiste già")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("creato e importato automaticamente.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("perché si riferisce allo stesso file.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Indicatore data e ora	Nomefile	Errore	Messaggio")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" avviata") || msg.ends_with(" avviate")
    }
}
