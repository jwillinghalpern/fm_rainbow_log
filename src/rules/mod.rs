use crate::ImportLogLine;

// import language modules here
mod de;
mod en;
mod es;
mod fr;
mod it;
mod ja;
mod ko;
mod nl;
mod pt;
mod sv;
mod zh;

fn get_rules_impls() -> &'static [&'static dyn Rules] {
    // be sure to add each language's RulesImpl:
    &[
        &en::RulesImpl,
        &de::RulesImpl,
        &es::RulesImpl,
        &fr::RulesImpl,
        &it::RulesImpl,
        &ja::RulesImpl,
        &ko::RulesImpl,
        &nl::RulesImpl,
        &pt::RulesImpl,
        &sv::RulesImpl,
        &zh::RulesImpl,
    ]
}

/// This trait defines all the methods that must be implemented for each language to comply with the parsing checker. Implement Rules for a unit-like struct in each language file.
trait Rules {
    // these methods must be implemented for each language
    fn warning_already_exists(&self, msg: &str) -> bool;
    fn warning_eds_created_and_imported_automatically(&self, msg: &str) -> bool;
    fn warning_eds_used_instead(&self, msg: &str) -> bool;

    fn is_header(&self, msg: &str) -> bool;
    fn is_operation_start(&self, msg: &str) -> bool;

    // this method has a blanket implementation. No need to reimplement.
    fn contains_warning_text(&self, msg: &str) -> bool {
        self.warning_already_exists(msg)
            || self.warning_eds_created_and_imported_automatically(msg)
            || self.warning_eds_used_instead(msg)
    }
}

pub(crate) fn contains_warning_text(line: &ImportLogLine) -> bool {
    get_rules_impls()
        .iter()
        .any(|rules| rules.contains_warning_text(&line.message))
}

pub(crate) fn is_operation_start(line: &ImportLogLine) -> bool {
    get_rules_impls()
        .iter()
        .any(|rules| rules.is_operation_start(&line.message))
}

pub(crate) fn is_header(line: &str) -> bool {
    get_rules_impls().iter().any(|rules| rules.is_header(line))
}
