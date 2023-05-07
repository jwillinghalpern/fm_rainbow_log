use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("이미 존재합니다.")
            || msg.ends_with("이미 존재합니다..") // I'm not sure if the trailing doubledot is a typo in the test data or not
            || msg.ends_with("”인 값 목록이 이미 존재함).")
            || msg.ends_with("”은(는) 이미 존재합니다).")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.contains("자동으로 생성되고 가져왔습니다.")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.contains("같은 파일을 참조하므로 대신 파일 참조")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("타임 스탬프	파일 이름	오류	메시지")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" 가져오기가 시작됨") || msg.ends_with("가져오기 작업 시작됨")
    }
}
