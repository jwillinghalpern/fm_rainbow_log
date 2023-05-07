use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.contains("名为 “")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.contains("自动创建并导入丢失的文件参考")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.contains("因为参考同一文件，所以使用文件参考")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("时间戳	文件名	错误	信息")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.starts_with("开始从剪贴板导") || msg.starts_with("导入操作已开始")
    }
}
