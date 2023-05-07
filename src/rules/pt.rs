use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("já existe.")
            || msg.contains("”, pois já existe uma função nomeada ")
            || msg.contains("já existe uma lista de valor com nome “")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.ends_with("criada e importada automaticamente.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.ends_with("foi usada, pois faz referência ao mesmo arquivo.")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("Carimbo de data/hora	Nome do arquivo	Erro	Mensagem")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" iniciada") || msg.ends_with(" iniciadas")
    }
}
