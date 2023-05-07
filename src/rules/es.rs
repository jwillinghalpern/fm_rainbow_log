use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("ya existe.")
            || msg.contains("” pues ya existe un")
            || msg.contains("” porque ya existe un")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("creada e importada automáticamente.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("ya que se refiere al mismo archivo.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Fecha y hora	Nombre de archivo	Error	Mensaje")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" iniciada")
    }
}
