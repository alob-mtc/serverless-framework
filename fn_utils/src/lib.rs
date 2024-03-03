pub mod template;

pub fn to_camel_case_handler(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in input.chars().enumerate() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next || i == 0 {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result.push_str("Handler");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case_handler() {
        assert_eq!(to_camel_case_handler("hello-world"), "HelloWorldHandler");
        assert_eq!(to_camel_case_handler("hello"), "HelloHandler");
        assert_eq!(
            to_camel_case_handler("hello-world-again"),
            "HelloWorldAgainHandler"
        );
    }
}
