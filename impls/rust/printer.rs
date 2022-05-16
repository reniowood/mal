use crate::types::MalType;

pub fn pr_str(value: &MalType) -> String {
    match value {
        MalType::True => "true".to_string(),
        MalType::False => "false".to_string(),
        MalType::Nil => "nil".to_string(),
        MalType::Number(number) => format!("{}", number),
        MalType::Keyword(name) => format!("{}", name),
        MalType::Symbol(name) => format!("{}", name),
        MalType::String(value) => format!("{}", value),
        MalType::List(list) => format!("({})", list.iter().map(|value| pr_str(value)).collect::<Vec<String>>().join(" ")),
        MalType::Quote(value) => format!("(quote {})", pr_str(value)),
        MalType::QuasiQuote(value) => format!("(quasiquote {})", pr_str(value)),
        MalType::Unquote(value) => format!("(unquote {})", pr_str(value)),
        MalType::SpliceUnquote(value) => format!("(splice-unquote {})", pr_str(value)),
        MalType::Hashmap(value) => format!("{{{}}}", value.iter().map(|(key, value)| format!("{} {}", key, pr_str(value))).collect::<Vec<String>>().join(" ")),
        MalType::Vector(list) => format!("[{}]", list.iter().map(|value| pr_str(value)).collect::<Vec<String>>().join(" ")),
        MalType::Deref(value) => format!("(deref {})", pr_str(value)),
        MalType::WithMeta(first, second) => format!("(with-meta {} {})", pr_str(second), pr_str(first)),
    }
}