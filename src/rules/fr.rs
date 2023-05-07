use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("existe déjà.")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("créée et importée automatiquement.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("utilisée en remplacement, car elle fait référence au même fichier.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Horodatage	NomFichier	Erreur	Message")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" démarrée") || msg.ends_with(" démarrées")
    }
}
