pub trait SubStrings {
    fn substr_before(self, delimiter: char) -> String;
    fn substr_after(self, delimiter: char) -> String;
    fn substr_before_last(self, delimiter: char) -> String;
    fn substr_after_last(self, delimiter: char) -> String;
    fn remove_surrounding(self, prefix: char, suffix: char) -> String;
}

impl SubStrings for String {
    fn substr_before(self, delimiter: char) -> String {
        let idx = self.find(delimiter).unwrap();
        self[0..idx].to_string()
    }

    fn substr_after(self, delimiter: char) -> String {
        let idx = self.find(delimiter).unwrap();
        self[idx + 1..self.len()].to_string()
    }

    fn substr_before_last(self, delimiter: char) -> String {
        let idx = self.rfind(delimiter).unwrap();
        self[0..idx].to_string()
    }

    fn substr_after_last(self, delimiter: char) -> String {
        let idx = self.rfind(delimiter).unwrap();
        self[idx + 1..self.len()].to_string()
    }

    fn remove_surrounding(self, prefix: char, suffix: char) -> String {
        if self.len() >= 2 && self.starts_with(prefix) && self.ends_with(suffix) {
            return self[1..self.len() - 1].to_string();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::sub_strings::SubStrings;

    #[test]
    fn test_substr_before() {
        let before = "hello world".to_string();
        let after = before.substr_before(' ');
        assert_eq!(after, "hello")
    }

    #[test]
    fn test_substr_after() {
        let before = "hello world".to_string();
        let after = before.substr_after(' ');
        assert_eq!(after, "world")
    }

    #[test]
    fn test_substr_before_last() {
        let before = "hello world".to_string();
        let after = before.substr_before_last('o');
        assert_eq!(after, "hello w")}

    #[test]
    fn test_substr_after_last() {
        let before = "hello world".to_string();
        let after = before.substr_after_last('o');
        assert_eq!(after, "rld")}

    #[test]
    fn test_remove_surrounding() {
        let before = "<hello world>".to_string();
        let after = before.remove_surrounding('<', '>');
        assert_eq!(after, "hello world")}

}
