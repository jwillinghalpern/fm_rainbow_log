use super::Rules;

pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("bereits existiert.")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("automatisch erstellt und importiert.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("stattdessen verwendet, da er sich auf die gleiche Datei bezieht.")
    }
    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Zeitstempel	Dateiname	Fehler	Meldung")
    }
    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" begonnen")
    }
}
