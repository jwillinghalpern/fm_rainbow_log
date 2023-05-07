use super::Rules;

// each language module should implement Rules on a unit-like struct:
pub(crate) struct RulesImpl;
impl Rules for RulesImpl {
    fn warning_already_exists(&self, msg: &str) -> bool {
        msg.ends_with("はすでに存在します。") || msg.ends_with("としてインポートされました。")
    }
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool {
        msg.contains("自動的に作成およびインポートされたファイル参照")
    }
    fn warning_eds_used_instead(&self, msg: &str) -> bool {
        msg.contains("同じファイルを参照しているため、ファイル参照")
    }

    fn is_header(&self, msg: &str) -> bool {
        msg.ends_with("タイムスタンプ	ファイル名	エラー	メッセージ")
    }

    fn is_operation_start(&self, msg: &str) -> bool {
        msg.ends_with(" のインポートを開始しました")
            || msg.ends_with("インポート処理が開始されました")
    }
}
